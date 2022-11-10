use std::{env, net::SocketAddr};

use actix_cors::Cors;
use actix_web::HttpResponse;
use actix_web::{post, web, App, HttpServer, Responder};
use dotenvy::dotenv;
use sendgrid_thin::Sendgrid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EmailBody {
    subject: String,
    body: String,
}

#[derive(Serialize)]
struct EmailSendResponse {
    message: String,
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[post("/send-mail")]
async fn send_email(req_body: web::Json<EmailBody>) -> impl Responder {
    let sendgrid_api_key = env::var("SENDGRID_API_KEY").unwrap();
    let sendgrid = Sendgrid::new(
        &sendgrid_api_key,
        "reinaldorozatoj@gmail.com",
        "reinaldorozatoj.11cg1@aleeas.com",
    );

    match sendgrid.send_mail(&req_body.subject, &req_body.body) {
        Ok(_) => HttpResponse::Ok().json(EmailSendResponse {
            message: "Email sent successfully".to_string(),
            success: true,
            error: None,
        }),
        Err(err) => HttpResponse::InternalServerError().json(EmailSendResponse {
            message: "Error sending email".to_string(),
            error: Some(err.to_string()),
            success: false,
        }),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT")
            .unwrap_or_else(|_| {
                println!("PORT not found .env file, using default port: 8080");
                "8080".to_string()
            })
            .parse::<u16>()
            .expect("Failed to parse port from .env file"),
    ));

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://www.oloko64.dev")
            .allowed_methods(vec!["GET", "POST"]);
        App::new()
            .wrap(cors)
            .route(
                "/",
                web::get().to(|| async {
                    "My custom email service for my website using Actix-Web and SendGrid"
                }),
            )
            .service(send_email)
    })
    .bind(&addr)?
    .run()
    .await
}
