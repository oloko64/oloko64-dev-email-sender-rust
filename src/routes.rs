use std::time::Duration;

use axum::Json;
use sendgrid_thin::Sendgrid;
use tracing::error;

use crate::{
    responses::{ApiError, EmailSentResponse},
    telegram::Telegram,
    utils::{self, config, EmailBody},
    REQUEST_TIMEOUT_SEC,
};

pub async fn send_message(Json(req_body): Json<EmailBody>) -> Result<EmailSentResponse, ApiError> {
    utils::validate_body(&req_body)?;

    let sendgrid_api_key = config().get_sendgrid_api_key();
    let from_email = config().get_send_from_email();
    let to_email = config().get_send_to_email();

    let message_body = format!(
        "Contact: {}\n\nMessage: {}",
        req_body.contact, req_body.body
    );

    let sendgrid = Sendgrid::builder(
        sendgrid_api_key,
        from_email,
        [to_email],
        &req_body.subject,
        &message_body,
    )
    .set_request_timeout(Duration::from_secs(REQUEST_TIMEOUT_SEC))
    .build()?;

    let (telegram_response, email_response) = tokio::join!(
        Telegram::send_notification(&req_body.subject, message_body),
        sendgrid.send()
    );
    let email_response_text = match email_response {
        Ok(response) => response.public_response,
        Err(_) => String::from("Error while sending email"),
    };

    let telegram_response = telegram_response.map_err(|_| {
        ApiError::internal_server_error(
            "Error while sending Telegram notification",
            "Something went wrong while sending Telegram notification",
        )
    })?;

    Ok(EmailSentResponse {
        email_message: email_response_text,
        telegram_message: telegram_response,
    })
}
