use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::{env::VarError, fmt};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum UserError {
    BadRequest { message: String, error: String },

    InternalServerError { message: String, error: String },
}

impl UserError {
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
        UserError::InternalServerError {
            message: message.into(),
            error: error.into(),
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self).expect("Failed to serialize response")
        )
    }
}

impl std::error::Error for UserError {}

impl From<VarError> for UserError {
    fn from(error: VarError) -> Self {
        UserError::InternalServerError {
            message: String::from("Error while getting environment variable"),
            error: error.to_string(),
        }
    }
}

impl From<&str> for UserError {
    fn from(error: &str) -> Self {
        UserError::BadRequest {
            message: String::from(error),
            error: String::from(error),
        }
    }
}

impl From<reqwest::Error> for UserError {
    fn from(error: reqwest::Error) -> Self {
        UserError::InternalServerError {
            message: String::from("Request error"),
            error: error.to_string(),
        }
    }
}

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            UserError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSentResponse {
    message: String,
}

impl EmailSentResponse {
    pub fn ok<T>(message: T) -> HttpResponse
    where
        T: Into<String>,
    {
        HttpResponse::Ok().json(EmailSentResponse {
            message: message.into(),
        })
    }
}
