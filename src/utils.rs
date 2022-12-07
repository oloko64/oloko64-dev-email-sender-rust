use std::{
    env::{self, VarError},
    net::SocketAddr,
};

use actix_web::HttpResponse;
use log::{error, warn};

use crate::responses::EmailSendResponse;

const DEFAULT_PORT: u16 = 8080;

pub fn get_env_variable(
    var: Result<String, VarError>,
    error_message: &str,
) -> Result<String, HttpResponse> {
    match var {
        Ok(value) => Ok(value),
        Err(_) => {
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
