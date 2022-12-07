use actix_cors::Cors;
use actix_web::{http, post, web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use lambda_web::{is_running_on_lambda, run_actix_on_lambda, LambdaError};
use log::{error, info, warn};
use sendgrid_thin::Sendgrid;
use serde::{Deserialize, Serialize};
use std::{
    env::{self, VarError},
    net::SocketAddr,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const DEFAULT_PORT: u16 = 8080;

#[derive(Deserialize)]
struct EmailBody {
    subject: String,
    body: String,
}

#[derive(Serialize)]
struct EmailSendResponse<'a> {
    message: &'a str,
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
}

fn get_env_variable(
    var: Result<String, VarError>,
    error_message: &str,
) -> Result<String, HttpResponse> {
    match var {
        Ok(value) => Ok(value),
        Err(_) => {
            error!("{}", error_message);
            sentry::capture_message(error_message, sentry::Level::Error);
            Err(HttpResponse::InternalServerError().json(EmailSendResponse {
                message: "Internal Server Error",
                success: false,
                error: Some(error_message),
            }))
        }
    }
}

fn get_socket_addr() -> SocketAddr {
    SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT")
            .unwrap_or_else(|_| {
                warn!(
                    "PORT not found .env file, using default port: {}",
                    DEFAULT_PORT
                );
                DEFAULT_PORT.to_string()
            })
            .parse::<u16>()
            .unwrap_or_else(|_| {
                warn!(
                    "PORT is not a valid port number, using default port: {}",
                    DEFAULT_PORT
                );
                DEFAULT_PORT
            }),
    ))
}

#[post("/send-mail")]
async fn send_email(req_body: web::Json<EmailBody>) -> impl Responder {
    let sendgrid_api_key =
        match get_env_variable(env::var("SENDGRID_API_KEY"), "SENDGRID_API_KEY not set") {
            Ok(value) => value,
            Err(http_response) => return http_response,
        };
    let from_email = match get_env_variable(env::var("SEND_FROM_EMAIL"), "SEND_FROM_EMAIL not set")
    {
        Ok(value) => value,
        Err(http_response) => return http_response,
    };
    let to_email = match get_env_variable(env::var("SEND_TO_EMAIL"), "SEND_TO_EMAIL not set") {
        Ok(value) => value,
        Err(http_response) => return http_response,
    };

    let mut sendgrid = Sendgrid::new(&sendgrid_api_key);
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
            HttpResponse::Ok().json(EmailSendResponse {
                message: &message,
                success: true,
                error: None,
            })
        }
        Err(err) => {
            sentry::capture_message(&err.to_string(), sentry::Level::Error);
            HttpResponse::BadRequest().json(EmailSendResponse {
                message: "Error sending email",
                error: Some(err.to_string().as_str()),
                success: false,
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
