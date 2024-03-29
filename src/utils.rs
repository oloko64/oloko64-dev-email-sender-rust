use serde::Deserialize;
use std::{
    env::{self, VarError},
    net::SocketAddr,
    sync::OnceLock,
};
use tracing::warn;
use unicode_segmentation::UnicodeSegmentation;

const PORT: u16 = 3000;

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

pub fn validate_body(body: &EmailBody) -> Result<(), &'static str> {
    if body.contact.is_empty() {
        return Err("Contact cannot be empty");
    }

    if body.subject.is_empty() {
        return Err("Subject cannot be empty");
    }

    if body.body.is_empty() {
        return Err("Body cannot be empty");
    }

    if body.contact.graphemes(true).count() > 50 {
        return Err("Contact cannot be longer than 50 characters");
    }

    if body.subject.graphemes(true).count() > 50 {
        return Err("Subject cannot be longer than 50 characters");
    }

    if body.body.graphemes(true).count() > 2000 {
        return Err("Body cannot be longer than 2000 characters");
    }

    Ok(())
}

pub fn get_socket_addr() -> SocketAddr {
    SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT")
            .unwrap_or_else(|_| {
                warn!("PORT not found .env file, using default port: {PORT}");
                PORT.to_string()
            })
            .parse::<u16>()
            .unwrap_or_else(|_| {
                warn!("PORT is not a valid port number, using default port: {PORT}");
                PORT
            }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_body_should_return_ok_when_valid() {
        let body = EmailBody {
            contact: String::from("Contact"),
            subject: String::from("Subject"),
            body: String::from("Body"),
        };

        assert_eq!(validate_body(&body), Ok(()));
    }

    #[test]
    fn validate_body_should_return_error_when_contact_is_empty() {
        let body = EmailBody {
            contact: String::new(),
            subject: String::from("Subject"),
            body: String::from("Body"),
        };

        assert_eq!(validate_body(&body), Err("Contact cannot be empty"));
    }

    #[test]
    fn validate_body_should_return_error_when_subject_is_empty() {
        let body = EmailBody {
            contact: String::from("Contact"),
            subject: String::new(),
            body: String::from("Body"),
        };

        assert_eq!(validate_body(&body), Err("Subject cannot be empty"));
    }

    #[test]
    fn validate_body_should_return_error_when_body_is_empty() {
        let body = EmailBody {
            contact: String::from("Contact"),
            subject: String::from("Subject"),
            body: String::new(),
        };

        assert_eq!(validate_body(&body), Err("Body cannot be empty"));
    }

    #[test]
    fn validate_body_should_return_error_when_contact_is_longer_than_50_characters() {
        let body = EmailBody {
            contact: "a".repeat(51),
            subject: String::from("Subject"),
            body: String::from("Body"),
        };

        assert_eq!(
            validate_body(&body),
            Err("Contact cannot be longer than 50 characters")
        );
    }

    #[test]
    fn validate_body_should_return_error_when_subject_is_longer_than_50_characters() {
        let body = EmailBody {
            contact: String::from("Contact"),
            subject: "a".repeat(51),
            body: String::from("Body"),
        };

        assert_eq!(
            validate_body(&body),
            Err("Subject cannot be longer than 50 characters")
        );
    }

    #[test]
    fn validate_body_should_return_error_when_body_is_longer_than_2000_characters() {
        let body = EmailBody {
            contact: String::from("Contact"),
            subject: String::from("Subject"),
            body: "a".repeat(2001),
        };

        assert_eq!(
            validate_body(&body),
            Err("Body cannot be longer than 2000 characters")
        );
    }
}
