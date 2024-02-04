use std::time::Duration;

use axum::Json;
use sendgrid_thin::Sendgrid;

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

    let sent_response = format!(
        "Email response -> {} | Telegram response -> {}",
        email_response.unwrap_or(String::from("Error while sending email")),
        telegram_response.map_err(|_| ApiError::internal_server_error(
            "Error while sending Telegram notification",
            "Something went wrong while sending Telegram notification"
        ))?
    );

    Ok(EmailSentResponse::new(sent_response))
}
