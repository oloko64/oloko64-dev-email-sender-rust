use std::fmt;

use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq, Eq))]
pub struct EmailSendResponse {
    message: String,

    #[serde(skip)]
    status_code: StatusCode,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl error::ResponseError for EmailSendResponse {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}

impl fmt::Display for EmailSendResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.json_string())
    }
}

impl EmailSendResponse {
    pub fn new<T, U>(status_code: StatusCode, message: T, error: Option<U>) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        Self {
            status_code,
            message: message.into(),
            error: error.map(std::convert::Into::into),
        }
    }

    pub fn create<T, U>(status_code: StatusCode, message: T, error: Option<U>) -> HttpResponse
    where
        T: Into<String>,
        U: Into<String>,
    {
        HttpResponse::Ok().json(Self {
            status_code,
            message: message.into(),
            error: error.map(std::convert::Into::into),
        })
    }

    fn json_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize response")
    }
}
