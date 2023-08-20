use actix_web::web;
use log::warn;
use serde::Deserialize;
use std::{env, net::SocketAddr};
use unicode_segmentation::UnicodeSegmentation;

use crate::responses::UserError;

const DEFAULT_PORT: u16 = 8080;

#[derive(Deserialize)]
pub struct EmailBody {
    pub contact: String,
    pub subject: String,
    pub body: String,
}

pub struct EnvVars;

impl EnvVars {
    pub fn get_telegram_bot_token() -> Result<String, UserError> {
        Ok(env::var("TELEGRAM_BOT_TOKEN")?)
    }

    pub fn get_telegram_chat_id() -> Result<String, UserError> {
        Ok(env::var("TELEGRAM_CHAT_ID")?)
    }

    pub fn get_sendgrid_api_key() -> Result<String, UserError> {
        Ok(env::var("SENDGRID_API_KEY")?)
    }

    pub fn get_send_from_email() -> Result<String, UserError> {
        Ok(env::var("SEND_FROM_EMAIL")?)
    }

    pub fn get_send_to_email() -> Result<String, UserError> {
        Ok(env::var("SEND_TO_EMAIL")?)
    }
}

pub fn validate_body(body: &web::Json<EmailBody>) -> Result<(), &'static str> {
    match body {
        _ if body.contact.is_empty() => Err("Contact cannot be empty"),
        _ if body.subject.is_empty() => Err("Subject cannot be empty"),
        _ if body.body.is_empty() => Err("Body cannot be empty"),
        _ if body.contact.graphemes(true).count() > 50 => {
            Err("Contact cannot be longer than 50 characters")
        }
        _ if body.subject.graphemes(true).count() > 50 => {
            Err("Subject cannot be longer than 50 characters")
        }
        _ if body.body.graphemes(true).count() > 2000 => {
            Err("Body cannot be longer than 2000 characters")
        }
        _ => Ok(()),
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
