use actix_cors::Cors;
use actix_web::{http, post, web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use lambda_web::{is_running_on_lambda, run_actix_on_lambda, LambdaError};
use log::error;
use sendgrid_thin::Sendgrid;
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr};

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

#[post("/send-mail")]
async fn send_email(req_body: web::Json<EmailBody>) -> impl Responder {
    let sendgrid_api_key = match env::var("SENDGRID_API_KEY") {
        Ok(api_key) => api_key,
        Err(err) => {
            sentry::capture_error(&err);
            return HttpResponse::InternalServerError().json(EmailSendResponse {
                message: "Internal Server Error",
                success: false,
                error: Some("Sendgrid API Key not found"),
            });
        }
    };
    let mut sendgrid = Sendgrid::new(&sendgrid_api_key);
    sendgrid
        .set_from_email("reinaldorozatoj.11cg1@aleeas.com")
        .set_to_emails(&["reinaldorozatoj@gmail.com"])
        .set_subject(&req_body.subject)
        .set_body(&req_body.body);

    match sendgrid.send() {
        Ok(message) => HttpResponse::Ok().json(EmailSendResponse {
            message: &message,
            success: true,
            error: None,
        }),
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
    dotenv().ok();
    env_logger::init();
    let sentry_api_key = env::var("SENTRY_API_KEY").unwrap_or_else(|_| {
        error!("Sentry API Key not found");
        String::new()
    });
    let _guard = sentry::init((
        sentry_api_key,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT")
            .unwrap_or_else(|_| {
                println!("PORT not found .env file, using default port: 8080");
                "8080".to_owned()
            })
            .parse::<u16>()
            .expect("Failed to parse port from .env file"),
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
        HttpServer::new(factory).bind(&addr)?.run().await?;
    }

    Ok(())
}
