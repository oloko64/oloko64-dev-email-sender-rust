use axum::{Json, http::StatusCode, response::IntoResponse};
use sendgrid_thin::SendgridError;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{error, warn};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ApiError {
    BadRequest { message: String, error: String },

    InternalServerError { message: String, error: String },
}

impl ApiError {
    pub fn bad_request<T, U>(message: T, error: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        ApiError::BadRequest {
            message: message.into(),
            error: error.into(),
        }
    }

    pub fn internal_server_error<T, U>(message: T, error: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        ApiError::InternalServerError {
            message: message.into(),
            error: error.into(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self).expect("Failed to serialize response")
        )
    }
}

impl std::error::Error for ApiError {}

impl From<&str> for ApiError {
    fn from(error: &str) -> Self {
        ApiError::bad_request(String::from(error), String::from(error))
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError::internal_server_error("Request error", error.to_string())
    }
}

impl From<SendgridError> for ApiError {
    fn from(error: SendgridError) -> Self {
        ApiError::internal_server_error("Error with email", error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError { .. } => {
                error!("{}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
            }
            ApiError::BadRequest { .. } => {
                // Client mistakes are not app errors, so keep them out of Sentry.
                warn!("{}", self);
                (StatusCode::BAD_REQUEST, Json(self)).into_response()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSentResponse {
    pub email_message: String,
    pub telegram_message: String,
}

impl IntoResponse for EmailSentResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
