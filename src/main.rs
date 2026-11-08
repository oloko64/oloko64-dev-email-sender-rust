mod request_log;
mod responses;
mod routes;
mod spam;
mod telegram;
mod utils;

use axum::{
    Json, Router,
    http::{HeaderValue, Method, header},
    middleware::from_fn,
    routing::{get, post},
};
use lambda_http::Error;
use lambda_runtime::tower::ServiceBuilder;
use sentry::integrations::tracing::{BreadcrumbLayer, ErrorLayer};
use serde_json::json;
use std::{
    env::{self, set_var},
    time,
};
use tower_http::{
    catch_panic::CatchPanicLayer, cors::CorsLayer, normalize_path::NormalizePathLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt, prelude::*};
use utils::get_socket_addr;

const REQUEST_TIMEOUT_SEC: u64 = 5;

fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    // If you use API Gateway stages, the Rust Runtime will include the stage name
    // as part of the path that your application receives.
    // Setting the following environment variable, you can remove the stage from the path.
    // This variable only applies to API Gateway stages,
    // you can remove it if you don't use them.
    // i.e with: `GET /test-stage/todo/id/123` without: `GET /todo/id/123`
    //
    // SAFETY: called before the Tokio runtime starts, so no other threads exist yet.
    unsafe {
        set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");
    }

    run_runtime()
}

#[tokio::main]
async fn run_runtime() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_target(false).with_ansi(false))
        // Sends `error!` events to Sentry as errors, with `info!`/`warn!` as breadcrumbs.
        .with(ErrorLayer::new().with_filter(LevelFilter::ERROR))
        .with(BreadcrumbLayer::new().with_filter(LevelFilter::INFO))
        .init();

    info!("App version: v{}", env!("CARGO_PKG_VERSION"));

    let sentry_api_key = env::var("SENTRY_API_KEY").unwrap_or_else(|_| {
        warn!("Sentry API Key not found, not reporting errors to Sentry");
        String::new()
    });
    // `ClientOptions` is `#[non_exhaustive]` since sentry 0.49, so it can't be built with a struct literal.
    let mut sentry_options = sentry::ClientOptions::new().attach_stacktrace(true);
    sentry_options.release = sentry::release_name!();
    let _guard = sentry::init((sentry_api_key, sentry_options));

    // Fail at startup instead of on the first request if the config is incomplete.
    utils::init_config()?;

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("https://www.oloko64.dev"))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let middlewares = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // After CORS so preflight requests aren't logged; before CatchPanic so panics are logged as 500.
        .layer(from_fn(request_log::request_logger))
        .layer(CatchPanicLayer::new())
        .layer(NormalizePathLayer::trim_trailing_slash());

    // API v1 routes
    let routes_v1 = Router::new()
        .route("/send-message", post(routes::send_message))
        .route("/health", get(health_check));

    let app = Router::new()
        .route("/", get(|| async { "Email sender" }))
        .nest("/v1", routes_v1)
        .route("/health", get(health_check))
        .layer(middlewares);

    // The Lambda runtime always sets this variable, so the same binary works in both environments.
    if env::var("AWS_LAMBDA_RUNTIME_API").is_ok() {
        info!("Starting Lambda runtime");
        lambda_http::run(app).await?;
    } else {
        let socket_addr = get_socket_addr();
        let tcp_listener = tokio::net::TcpListener::bind(&socket_addr).await?;
        info!("Listening on {socket_addr}");
        axum::serve(tcp_listener, app).await?;
    }

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
    let server_time = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    Json(json!({ "status": "ok", "version": APP_VERSION, "timestamp": server_time }))
}
