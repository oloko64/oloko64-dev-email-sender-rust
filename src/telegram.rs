use serde_json::json;

use crate::{responses::ApiError, utils::config, REQUEST_TIMEOUT_SEC};

pub struct Telegram;

impl Telegram {
    pub async fn send_notification<T, U>(subject: U, message: T) -> Result<String, ApiError>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let bot_token = config().get_telegram_bot_token();
        let chat_id = config().get_telegram_chat_id();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SEC))
            .build()?;

        let body = json!({
            "chat_id": chat_id,
            "text": format!("Subject: {}\n\n{}", subject.as_ref(), message.as_ref()),
        });

        let response = client
            .post(format!(
                "https://api.telegram.org/bot{bot_token}/sendMessage"
            ))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ApiError::internal_server_error(
                format!(
                    "Error sending Telegram notification, request status {}",
                    response.status()
                ),
                response.text().await?,
            ));
        }

        Ok(String::from("Telegram notification sent successfully"))
    }
}
