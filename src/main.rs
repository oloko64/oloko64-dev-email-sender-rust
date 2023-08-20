mod responses;
mod telegram;
mod utils;

use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpRequest, HttpServer, Responder, Result};
use dotenvy::dotenv;
use lambda_web::{is_running_on_lambda, run_actix_on_lambda, LambdaError};
use log::{error, info, warn};
use sendgrid_thin::Sendgrid;
use std::env::{self, set_var};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utils::{get_socket_addr, EmailBody, EnvVars};

use crate::{
    responses::{EmailSentResponse, UserError},
    telegram::Telegram,
};

const REQUEST_TIMEOUT_SEC: u64 = 5;

async fn send_message(
    req: HttpRequest,
    req_body: web::Json<EmailBody>,
) -> Result<impl Responder, UserError> {
    info!("Client IP: {:?}", req.connection_info().peer_addr());
    utils::validate_body(&req_body)?;

    let sendgrid_api_key = EnvVars::get_sendgrid_api_key()?;
    let from_email = EnvVars::get_send_from_email()?;
    let to_email = EnvVars::get_send_to_email()?;

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
    .set_request_timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SEC))
    .build()
    .map_err(|err| {
        error!("Error while building Sendgrid: {err}");
        UserError::internal_server_error("Error while building Sendgrid", err.to_string())
    })?;

    let (telegram_response, email_response) = tokio::join!(
        Telegram::send_notification(&req_body.subject, message_body),
        sendgrid.send()
    );

    let sent_response = format!(
        "Email response -> {} | Telegram response -> {}",
        email_response.unwrap_or(String::from("Error while sending email")),
        telegram_response.map_err(|_| UserError::internal_server_error(
            "Error while sending Telegram notification",
            "Something went wrong while sending Telegram notification"
        ))?
    );

    info!("Message sent with subject: {}", req_body.subject);
    info!("{}", &sent_response);
    Ok(EmailSentResponse::ok(sent_response))
}

#[actix_web::main]
async fn main() -> Result<(), LambdaError> {
    dotenv().ok();
    if env::var("RUST_LOG").is_err() {
        set_var("RUST_LOG", "info");
    }

    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(false))
        .with(EnvFilter::from_default_env())
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
            .route("/send-message", web::post().to(send_message))
    };

    info!("App version: v{}", env!("CARGO_PKG_VERSION"));

    if is_running_on_lambda() {
        // Run on AWS Lambda
        run_actix_on_lambda(factory).await?;
    } else {
        // Run on a normal HTTP server
        HttpServer::new(factory)
            .bind(get_socket_addr())?
            .run()
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header, test, web::Bytes, App};

    #[actix_web::test]
    async fn test_missing_var_error() {
        let app =
            test::init_service(App::new().route("/send-message", web::post().to(send_message)))
                .await;
        let req = test::TestRequest::post()
            .uri("/send-message")
            .insert_header(header::ContentType::json())
            .set_payload(
                r#"{"contact": "Test contact!", "subject": "Test subject!", "body": "Test body!"}"#,
            )
            .to_request();
        let resp = test::call_and_read_body(&app, req).await;

        assert_eq!(resp, Bytes::from_static(br#"{"message":"Error while getting environment variable","error":"environment variable not found"}"#));
    }

    #[actix_web::test]
    async fn test_empty_contact_error() {
        let app =
            test::init_service(App::new().route("/send-message", web::post().to(send_message)))
                .await;
        let req = test::TestRequest::post()
            .uri("/send-message")
            .insert_header(header::ContentType::json())
            .set_payload(r#"{"contact": "", "subject": "Test subject!", "body": "Test body!"}"#)
            .to_request();
        let resp = test::call_and_read_body(&app, req).await;

        assert_eq!(
            resp,
            Bytes::from_static(
                br#"{"message":"Contact cannot be empty","error":"Contact cannot be empty"}"#
            )
        );
    }

    #[actix_web::test]
    async fn test_empty_subject_error() {
        let app =
            test::init_service(App::new().route("/send-message", web::post().to(send_message)))
                .await;
        let req = test::TestRequest::post()
            .uri("/send-message")
            .insert_header(header::ContentType::json())
            .set_payload(r#"{"contact": "Test contact!", "subject": "", "body": "Test body!"}"#)
            .to_request();
        let resp = test::call_and_read_body(&app, req).await;

        assert_eq!(
            resp,
            Bytes::from_static(
                br#"{"message":"Subject cannot be empty","error":"Subject cannot be empty"}"#
            )
        );
    }

    #[actix_web::test]
    async fn test_empty_body_error() {
        let app =
            test::init_service(App::new().route("/send-message", web::post().to(send_message)))
                .await;
        let req = test::TestRequest::post()
            .uri("/send-message")
            .insert_header(header::ContentType::json())
            .set_payload(r#"{"contact": "Test contact!", "subject": "Test subject!", "body": ""}"#)
            .to_request();
        let resp = test::call_and_read_body(&app, req).await;

        assert_eq!(
            resp,
            Bytes::from_static(
                br#"{"message":"Body cannot be empty","error":"Body cannot be empty"}"#
            )
        );
    }

    #[actix_web::test]
    async fn test_over_max_contact_size() {
        let app =
            test::init_service(App::new().route("/send-message", web::post().to(send_message)))
                .await;
        let req = test::TestRequest::post()
            .uri("/send-message")
            .insert_header(header::ContentType::json())
            .set_payload(
                r#"{"contact": "123456789012345678901234567890123456789012345678901", "subject": "Test subject!", "body": "Test body!"}"#,
            )
            .to_request();
        let resp = test::call_and_read_body(&app, req).await;

        assert_eq!(
            resp,
            Bytes::from_static(br#"{"message":"Contact cannot be longer than 50 characters","error":"Contact cannot be longer than 50 characters"}"#)
        );
    }

    #[actix_web::test]
    async fn test_over_max_subject_size() {
        let app =
            test::init_service(App::new().route("/send-message", web::post().to(send_message)))
                .await;
        let req = test::TestRequest::post()
            .uri("/send-message")
            .insert_header(header::ContentType::json())
            .set_payload(
                r#"{"contact": "Test contact!", "subject": "123456789012345678901234567890123456789012345678901", "body": "Test body!"}"#,
            )
            .to_request();
        let resp = test::call_and_read_body(&app, req).await;

        assert_eq!(
            resp,
            Bytes::from_static(br#"{"message":"Subject cannot be longer than 50 characters","error":"Subject cannot be longer than 50 characters"}"#)
        );
    }
}
