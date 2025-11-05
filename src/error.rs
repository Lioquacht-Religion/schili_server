// error.rs

use std::fmt::Display;

use actix_web::http::{self, header::ContentType};

#[derive(Debug)]
pub struct ApiError {
    error: anyhow::Error,
    status_code: http::StatusCode,
}

impl ApiError {
    pub fn new(status_code: http::StatusCode, error: anyhow::Error) -> Self {
        Self { error, status_code }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(value: anyhow::Error) -> Self {
        ApiError {
            error: value,
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error.to_string())
    }
}

impl actix_web::error::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.status_code
    }
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::build(self.status_code)
            .insert_header(ContentType::plaintext())
            .body(self.error.to_string())
    }
}
