// error.rs

use std::fmt::Display;

use actix_web::http::{self, header::ContentType};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ApiError {
    error: anyhow::Error,
    status_code: http::StatusCode,
}

#[derive(Debug)]
pub struct DateRangeError {
    error: String,
}

impl std::error::Error for DateRangeError {}

impl Display for DateRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl From<(Option<DateTime<Utc>>, Option<DateTime<Utc>>)> for DateRangeError {
    fn from(value: (Option<DateTime<Utc>>, Option<DateTime<Utc>>)) -> Self {
        let mut error = String::from("Invalid date range was supplied.\n");
        if let None = value.0 {
            error.push_str(&format!("Start date is invalid;\n"));
        }
        if let None = value.1 {
            error.push_str(&format!("End date is invalid;\n"));
        }
        if let (Some(start), Some(end)) = value {
            if start > end {
                error.push_str(&format!(
                    "Start date with value='{start}' is after
                        end date with value='{end}';\n"
                ));
            }
        }
        Self { error }
    }
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
