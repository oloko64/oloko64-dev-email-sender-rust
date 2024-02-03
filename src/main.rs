mod responses;
mod telegram;
mod utils;

use axum::{
    http::{header, Method},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dotenvy::dotenv;
use lambda_http::Error;
use lambda_runtime::tower::ServiceBuilder;
use sendgrid_thin::Sendgrid;
use std::{
    env::{self, set_var},
    time::Duration,
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utils::{config, get_socket_addr, EmailBody};

#[cfg(not(debug_assertions))]
use lambda_http::run;

use crate::{
    responses::{ApiError, EmailSentResponse},
    telegram::Telegram,
};

const REQUEST_TIMEOUT_SEC: u64 = 5;

async fn send_message(Json(req_body): Json<EmailBody>) -> Result<impl IntoResponse, ApiError> {
    utils::validate_body(&req_body)?;

    let sendgrid_api_key = config().get_sendgrid_api_key();
    let from_email = config().get_send_from_email();
    let to_email = config().get_send_to_email();

    let message_body = format!(
        "Contact: {}\n\nMessage: {}",
        req_body.contact, req_body.body
    );

    let sendgrid = Sendgrid::builder(
        sendgrid_api_key,
        from_email,
        [to_email],
        &req_body.subject,
        &message_body,
    )
    .set_request_timeout(Duration::from_secs(REQUEST_TIMEOUT_SEC))
    .build()?;

    let (telegram_response, email_response) = tokio::join!(
        Telegram::send_notification(&req_body.subject, message_body),
        sendgrid.send()
    );

    let sent_response = format!(
        "Email response -> {} | Telegram response -> {}",
        email_response.unwrap_or(String::from("Error while sending email")),
        telegram_response.map_err(|_| ApiError::internal_server_error(
            "Error while sending Telegram notification",
            "Something went wrong while sending Telegram notification"
        ))?
    );

    info!(
        "Message sent with subject: {} | Data: {}",
        req_body.subject, &sent_response
    );

    Ok(EmailSentResponse::ok(sent_response))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    // If you use API Gateway stages, the Rust Runtime will include the stage name
    // as part of the path that your application receives.
    // Setting the following environment variable, you can remove the stage from the path.
    // This variable only applies to API Gateway stages,
    // you can remove it if you don't use them.
    // i.e with: `GET /test-stage/todo/id/123` without: `GET /todo/id/123`
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    info!("App version: v{}", env!("CARGO_PKG_VERSION"));

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false))
        .with(fmt::layer().with_ansi(false))
        .init();

    let sentry_api_key = env::var("SENTRY_API_KEY").unwrap_or_else(|_| {
        warn!("Sentry API Key not found, not reporting errors to Sentry");
        String::new()
    });
    let _guard = sentry::init((
        sentry_api_key,
        sentry::ClientOptions {
            attach_stacktrace: true,
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let middlewares = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(CatchPanicLayer::new())
        .layer(NormalizePathLayer::trim_trailing_slash());

    let app = Router::new()
        .route("/", get(|| async { "Email sender" }))
        .route("/send-message", post(send_message))
        .layer(middlewares);

    #[cfg(debug_assertions)]
    {
        let socket_addr = get_socket_addr();
        let tcp_listener = tokio::net::TcpListener::bind(&socket_addr).await.unwrap();
        axum::serve(tcp_listener, app.into_make_service())
            .await
            .unwrap();
    }

    #[cfg(not(debug_assertions))]
    run(app).await?;

    Ok(())
}
