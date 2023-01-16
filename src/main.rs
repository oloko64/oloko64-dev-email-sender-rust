mod responses;
mod utils;

use std::env;

use actix_cors::Cors;
use actix_web::{http, post, web, App, HttpServer, Responder, Result};
use dotenvy::dotenv;
use lambda_web::{is_running_on_lambda, run_actix_on_lambda, LambdaError};
use log::{info, warn};
use sendgrid_thin::Sendgrid;
use serde::Deserialize;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utils::{get_socket_addr, EnvVars};

use crate::responses::{EmailSendResponse, UserError};

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

    match sendgrid.send() {
        Ok(message) => {
            info!(
                "Message sent: {:?} | subject: {}",
                message, req_body.subject
            );
            Ok(EmailSendResponse::ok(message))
        }
        Err(err) => {
            sentry::capture_message(&err.to_string(), sentry::Level::Error);
            Err(UserError::InternalServerError {
                body: EmailSendResponse::error(&err.to_string(), Some(&err.to_string())),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> Result<(), LambdaError> {
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
