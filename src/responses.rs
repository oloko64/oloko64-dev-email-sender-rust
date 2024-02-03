use axum::{http::StatusCode, response::IntoResponse, Json};
use sendgrid_thin::SendgridError;
use serde::{Deserialize, Serialize};
use std::{env::VarError, fmt};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ApiError {
    BadRequest { message: String, error: String },

    InternalServerError { message: String, error: String },
}

impl ApiError {
    // pub fn bad_request<T, U>(message: T, error: U) -> Self
    // where
    //     T: Into<String>,
    //     U: Into<String>,
    // {
    //     UserError::BadRequest {
    //         message: message.into(),
    //         error: error.into(),
    //     }
    // }

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

impl From<VarError> for ApiError {
    fn from(error: VarError) -> Self {
        ApiError::InternalServerError {
            message: String::from("Error while getting environment variable"),
            error: error.to_string(),
        }
    }
}

impl From<&str> for ApiError {
    fn from(error: &str) -> Self {
        ApiError::BadRequest {
            message: String::from(error),
            error: String::from(error),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError::InternalServerError {
            message: String::from("Request error"),
            error: error.to_string(),
        }
    }
}

impl From<SendgridError> for ApiError {
    fn from(error: SendgridError) -> Self {
        ApiError::InternalServerError {
            message: String::from("Error with email"),
            error: error.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            ApiError::BadRequest { .. } => (StatusCode::BAD_REQUEST, Json(self)).into_response(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSentResponse {
    message: String,
}

impl EmailSentResponse {
    pub fn ok<T>(message: T) -> axum::response::Json<EmailSentResponse>
    where
        T: Into<String>,
    {
        axum::response::Json(EmailSentResponse {
            message: message.into(),
        })
    }
}
