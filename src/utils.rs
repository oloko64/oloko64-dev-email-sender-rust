use crate::responses::UserError;
use log::warn;
use std::{env, net::SocketAddr};

const DEFAULT_PORT: u16 = 8080;

pub struct EnvVars;

impl EnvVars {
    pub fn get_sendgrid_api_key() -> Result<String, UserError> {
        Self::get_env_variable("SENDGRID_API_KEY")
    }

    pub fn get_send_from_email() -> Result<String, UserError> {
        Self::get_env_variable("SEND_FROM_EMAIL")
    }

    pub fn get_send_to_email() -> Result<String, UserError> {
        Self::get_env_variable("SEND_TO_EMAIL")
    }

    fn get_env_variable(env_variable: &str) -> Result<String, UserError> {
        let error_message = "Required env variable not set";
        let env_value = env::var(env_variable).map_err(|_| UserError::InternalServerError {
            message: String::from(error_message),
            error: String::from(error_message),
        })?;

        Ok(env_value)
    }
}

pub fn get_socket_addr() -> SocketAddr {
    SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT")
            .unwrap_or_else(|_| {
                warn!("PORT not found .env file, using default port: {DEFAULT_PORT}");
                DEFAULT_PORT.to_string()
            })
            .parse::<u16>()
            .unwrap_or_else(|_| {
                warn!("PORT is not a valid port number, using default port: {DEFAULT_PORT}");
                DEFAULT_PORT
            }),
    ))
}
