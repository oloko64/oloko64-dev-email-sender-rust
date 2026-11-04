use std::time::Duration;

use axum::{Json, extract::rejection::JsonRejection};
use sendgrid_thin::Sendgrid;
use tracing::error;

use crate::{
    REQUEST_TIMEOUT_SEC,
    responses::{ApiError, EmailSentResponse},
    telegram::Telegram,
    utils::{self, EmailBody, config},
};

const EMAIL_ERROR: &str = "Error while sending email";
const TELEGRAM_ERROR: &str = "Error while sending Telegram notification";

pub async fn send_message(
    req_body: Result<Json<EmailBody>, JsonRejection>,
) -> Result<EmailSentResponse, ApiError> {
    let Json(req_body) =
        req_body.map_err(|err| ApiError::bad_request("Invalid request body", err.body_text()))?;

    utils::validate_body(&req_body)?;

    let config = config();

    let message_body = format!(
        "Contact: {}\n\nMessage: {}",
        req_body.contact, req_body.body
    );

    let sendgrid = Sendgrid::builder(
        config.get_sendgrid_api_key(),
        config.get_send_from_email(),
        [config.get_send_to_email()],
        &req_body.subject,
        &message_body,
    )
    .set_request_timeout(Duration::from_secs(REQUEST_TIMEOUT_SEC))
    .build()
    .map_err(|err| {
        // Log the real error for Sentry, but don't expose it to the client.
        error!(error = %err, "Error while building email");
        ApiError::internal_server_error(EMAIL_ERROR, EMAIL_ERROR)
    })?;

    let (telegram_response, email_response) = tokio::join!(
        Telegram::send_notification(&req_body.subject, &message_body),
        sendgrid.send()
    );

    match (email_response, telegram_response) {
        (Err(email_err), Err(telegram_err)) => {
            error!(error = %email_err, "{EMAIL_ERROR}");
            error!(error = %telegram_err, "{TELEGRAM_ERROR}");
            Err(ApiError::internal_server_error(
                "Both email and Telegram notification failed",
                format!("{EMAIL_ERROR}; {TELEGRAM_ERROR}"),
            ))
        }
        (email_response, telegram_response) => Ok(EmailSentResponse {
            email_message: email_response.map_or_else(
                |err| {
                    error!(error = %err, "{EMAIL_ERROR}");
                    String::from(EMAIL_ERROR)
                },
                |response| response.public_response,
            ),
            telegram_message: telegram_response.unwrap_or_else(|err| {
                error!(error = %err, "{TELEGRAM_ERROR}");
                String::from(TELEGRAM_ERROR)
            }),
        }),
    }
}
