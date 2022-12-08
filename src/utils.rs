use std::{env, net::SocketAddr};

use actix_web::HttpResponse;
use log::{error, warn};

use crate::responses::EmailSendResponse;

const DEFAULT_PORT: u16 = 8080;

pub struct EnvVars {}

impl EnvVars {
    pub fn get_sendgrid_api_key() -> Result<String, HttpResponse> {
        Self::get_env_variable("SENDGRID_API_KEY", "SENDGRID_API_KEY not set")
    }

    pub fn get_send_from_email() -> Result<String, HttpResponse> {
        Self::get_env_variable("SEND_FROM_EMAIL", "SEND_FROM_EMAIL not set")
    }

    pub fn get_send_to_email() -> Result<String, HttpResponse> {
        Self::get_env_variable("SEND_TO_EMAIL", "SEND_TO_EMAIL not set")
    }

    pub fn get_env_variable(
        env_variable: &str,
        error_message: &str,
    ) -> Result<String, HttpResponse> {
        if let Ok(value) = env::var(env_variable) {
            Ok(value)
        } else {
            error!("{}", error_message);
            sentry::capture_message(error_message, sentry::Level::Error);
            Err(EmailSendResponse::internal_server_error(
                "Internal Server Error",
                Some(error_message),
            ))
        }
    }
}

pub fn get_socket_addr() -> SocketAddr {
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
