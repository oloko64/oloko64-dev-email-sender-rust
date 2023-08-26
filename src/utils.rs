use actix_web::web;
use log::warn;
use serde::Deserialize;
use std::{
    env::{self, VarError},
    net::SocketAddr,
    sync::OnceLock,
};
use unicode_segmentation::UnicodeSegmentation;

const DEFAULT_PORT: u16 = 8080;

#[derive(Deserialize)]
pub struct EmailBody {
    pub contact: String,
    pub subject: String,
    pub body: String,
}

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|err| {
            panic!("Error while loading config from environment variables: {err}")
        })
    })
}

pub struct Config {
    telegram_bot_token: String,
    telegram_chat_id: String,
    sendgrid_api_key: String,
    send_from_email: String,
    send_to_email: String,
}

impl Config {
    fn load_from_env() -> Result<Config, VarError> {
        Ok(Config {
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN")?,
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID")?,
            sendgrid_api_key: env::var("SENDGRID_API_KEY")?,
            send_from_email: env::var("SEND_FROM_EMAIL")?,
            send_to_email: env::var("SEND_TO_EMAIL")?,
        })
    }

    pub fn get_telegram_bot_token(&self) -> &str {
        &self.telegram_bot_token
    }

    pub fn get_telegram_chat_id(&self) -> &str {
        &self.telegram_chat_id
    }

    pub fn get_sendgrid_api_key(&self) -> &str {
        &self.sendgrid_api_key
    }

    pub fn get_send_from_email(&self) -> &str {
        &self.send_from_email
    }

    pub fn get_send_to_email(&self) -> &str {
        &self.send_to_email
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
