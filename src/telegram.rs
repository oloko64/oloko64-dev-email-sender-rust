use std::{sync::OnceLock, time::Duration};

use serde_json::json;

use crate::{REQUEST_TIMEOUT_SEC, responses::ApiError, utils::config};

/// Shared client so connections are reused across requests (and warm Lambda invocations).
fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SEC))
            .build()
            .expect("Failed to build HTTP client")
    })
}

pub struct Telegram;

impl Telegram {
    pub async fn send_notification<T, U>(subject: U, message: T) -> Result<String, ApiError>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        Self::send_text(format!(
            "Subject: {}\n\n{}",
            subject.as_ref(),
            message.as_ref()
        ))
        .await
    }

    /// Sends `text` as a plain-text message (no `parse_mode`, so nothing needs escaping).
    pub async fn send_text<T>(text: T) -> Result<String, ApiError>
    where
        T: AsRef<str>,
    {
        let bot_token = config().get_telegram_bot_token();
        let chat_id = config().get_telegram_chat_id();

        let body = json!({
            "chat_id": chat_id,
            "text": text.as_ref(),
        });

        let response = http_client()
            .post(format!(
                "https://api.telegram.org/bot{bot_token}/sendMessage"
            ))
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(ApiError::internal_server_error(
                format!("Error sending Telegram notification, request status {status}"),
                response.text().await?,
            ));
        }

        Ok(String::from("Telegram notification sent successfully"))
    }
}
