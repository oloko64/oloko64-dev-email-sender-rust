use crate::responses::UserError;
use actix_web::web;
use log::warn;
use serde::Deserialize;
use std::{env, net::SocketAddr};
use unicode_segmentation::UnicodeSegmentation;

const DEFAULT_PORT: u16 = 8080;

#[derive(Deserialize)]
pub struct EmailBody {
    pub subject: String,
    pub body: String,
}

pub struct EnvVars;

impl EnvVars {
    pub fn get_telegram_bot_token() -> Result<String, UserError> {
        Self::get_env_variable("TELEGRAM_BOT_TOKEN")
    }

    pub fn get_telegram_chat_id() -> Result<String, UserError> {
        Self::get_env_variable("TELEGRAM_CHAT_ID")
    }

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

pub fn validate_body(body: &web::Json<EmailBody>) -> Result<(), String> {
    if body.subject.is_empty() {
        return Err(String::from("Subject cannot be empty"));
    }

    if body.body.is_empty() {
        return Err(String::from("Body cannot be empty"));
    }

    if body.subject.graphemes(true).count() > 50 {
        return Err(String::from("Subject cannot be longer than 50 characters"));
    }

    if body.body.graphemes(true).count() > 2000 {
        return Err(String::from("Body cannot be longer than 2000 characters"));
    }

    Ok(())
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
