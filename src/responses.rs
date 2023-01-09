use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[cfg_attr(test, derive(Deserialize, Debug, PartialEq, Eq))]
pub struct EmailSendResponse {
    message: String,
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl EmailSendResponse {
    pub(crate) fn new(message: String, success: bool, error: Option<String>) -> Self {
        Self {
            message,
            success,
            error,
        }
    }
    pub fn ok(message: String) -> HttpResponse {
        HttpResponse::Ok().json(Self {
            message,
            success: true,
            error: None,
        })
    }

    pub fn internal_server_error<T>(message: T, error: Option<T>) -> HttpResponse
    where
        T: Into<String>,
    {
        HttpResponse::InternalServerError().json(Self {
            message: message.into(),
            success: false,
            error: error.map(|e| e.into()),
        })
    }

    pub fn bad_request<T>(message: T, error: Option<T>) -> HttpResponse
    where
        T: Into<String>,
    {
        HttpResponse::BadRequest().json(Self {
            message: message.into(),
            success: false,
            error: error.map(|e| e.into()),
        })
    }
}
