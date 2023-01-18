mod responses;
mod utils;

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, post, web, App, HttpServer, Responder, Result};
use dotenvy::dotenv;
use lambda_web::{is_running_on_lambda, run_actix_on_lambda, LambdaError};
use log::{info, warn};
use sendgrid_thin::Sendgrid;
use serde::Deserialize;
use std::env::{self, set_var};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utils::{get_socket_addr, EnvVars};

use crate::responses::{UserError, EmailSentResponse};

#[derive(Deserialize)]
struct EmailBody {
    subject: String,
    body: String,
}

#[post("/send-mail")]
async fn send_email(req_body: web::Json<EmailBody>) -> Result<impl Responder, UserError> {
    let sendgrid_api_key = EnvVars::get_sendgrid_api_key()?;
    let from_email = EnvVars::get_send_from_email()?;
    let to_email = EnvVars::get_send_to_email()?;

    let mut sendgrid = Sendgrid::new(sendgrid_api_key);
    sendgrid
        .set_from_email(from_email)
        .set_to_emails([to_email])
        .set_subject(&req_body.subject)
        .set_body(&req_body.body);

    let response_message = sendgrid
        .send()
        .map_err(|err| UserError::InternalServerError {
            message: "Error sending email".to_string(), error: err.to_string(),
        })?;

    info!(
        "Message sent: {} | subject: {}",
        response_message, req_body.subject
    );
    Ok(EmailSentResponse::ok(response_message))
}

#[actix_web::main]
async fn main() -> Result<(), LambdaError> {
    set_var("RUST_LOG", "info");

    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(false))
        .with(EnvFilter::from_default_env())
        .init();
    dotenv().ok();
    let sentry_api_key = env::var("SENTRY_API_KEY").unwrap_or_else(|_| {
        warn!("Sentry API Key not found, not reporting errors to Sentry");
        String::new()
    });
    let _guard = sentry::init((
        sentry_api_key,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let factory = move || {
        App::new()
            .wrap(sentry_actix::Sentry::new())
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("https://www.oloko64.dev")
                    .allowed_header(http::header::CONTENT_TYPE)
                    .allowed_methods(vec!["GET", "POST"]),
            )
            .route(
                "/",
                web::get().to(|| async {
                    "My custom email service for my website using Actix-Web and SendGrid"
                }),
            )
            .service(send_email)
    };

    info!("App version: v{}", env!("CARGO_PKG_VERSION"));

    if is_running_on_lambda() {
        // Run on AWS Lambda
        run_actix_on_lambda(factory).await?;
    } else {
        // Run on a normal HTTP server
        HttpServer::new(factory)
            .bind(&get_socket_addr())?
            .run()
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header, test, App, web::Bytes};

    #[actix_web::test]
    async fn test_missing_var_error() {
        let app = test::init_service(App::new().service(send_email)).await;
        let req = test::TestRequest::post()
            .uri("/send-mail")
            .insert_header(header::ContentType::json())
            .set_payload(r#"{"subject": "Test subject!", "body": "Test body!"}"#)
            .to_request();
        let resp = test::call_and_read_body(&app, req).await;

        assert_eq!(
            resp,
            Bytes::from_static(br#"{"message":"Required env variable not set","error":"Required env variable not set"}"#)
        );
    }
}
