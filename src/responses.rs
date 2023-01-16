use std::fmt;

use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use serde::Serialize;

#[derive(Debug, Display, Error)]
pub enum UserError {
    #[display(fmt = "{body}")]
    BadRequest { body: EmailSendResponse },

    #[display(fmt = "{body}")]
    InternalServerError { body: EmailSendResponse },
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

#[derive(Debug, Serialize)]
pub struct EmailSendResponse {
    message: String,
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl fmt::Display for EmailSendResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.json_string())
    }
}

impl EmailSendResponse {
    fn json_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize response")
    }

    pub fn ok(message: impl Into<String>) -> HttpResponse {
        HttpResponse::Ok().json(Self {
            message: message.into(),
            success: true,
            error: None,
        })
    }

    pub fn error<T>(message: T, error: Option<T>) -> Self
    where
        T: Into<String>,
    {
        Self {
            message: message.into(),
            success: false,
            error: error.map(Into::into),
        }
    }
}
