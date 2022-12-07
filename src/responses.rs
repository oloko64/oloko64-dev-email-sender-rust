use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct EmailSendResponse<'a> {
    message: &'a str,
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
}

impl<'a> EmailSendResponse<'a> {
    pub fn ok(message: &'a str) -> HttpResponse {
        HttpResponse::Ok().json(Self {
            message,
            success: true,
            error: None,
        })
    }

    pub fn internal_server_error(message: &'a str, error: Option<&'a str>) -> HttpResponse {
        HttpResponse::InternalServerError().json(Self {
            message,
            success: false,
            error,
        })
    }

    pub fn bad_request(message: &'a str, error: Option<&'a str>) -> HttpResponse {
        HttpResponse::BadRequest().json(Self {
            message,
            success: false,
            error,
        })
    }
}
