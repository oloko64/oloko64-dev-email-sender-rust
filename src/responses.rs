use std::fmt;

use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::Display;
use serde::{Serialize, Deserialize};

#[derive(Debug, Display)]
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

pub struct EmailSendResponseTest {
    pub message: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq, Eq))]
pub struct EmailSendResponse {
    pub(crate) message: String,
    pub(crate) success: bool,
    #[serde(skip)]
    pub(crate) status_code: StatusCode,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
}

impl error::ResponseError for EmailSendResponse {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code)
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
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
            status_code: StatusCode::OK,
            message: message.into(),
            success: true,
            error: None,
        })
    }

    pub fn error<T, U>(message: T, error: Option<U>) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
            success: false,
            error: error.map(Into::into),
        }
    }
}
