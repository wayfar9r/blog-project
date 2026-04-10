use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError, http::StatusCode, web::Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub error: String,
    pub details: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{error: {}, details: {} }}", self.error, self.details)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("validation error: {}. details: {}", .0.error, .0.details)]
    Validation(ValidationError),
    #[error("user not found: {0}")]
    NotFound(String),
    #[error("user already exists: {0}")]
    UserAlreadyExists(String),
    #[error("invalid credentials: {0}")]
    InvalidCredentials(String),
    #[error("unexpected failure: {0}")]
    UnexpectedFailure(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Json<String>>,
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            Self::UserAlreadyExists(_) => StatusCode::CONFLICT,
            Self::UnexpectedFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let (message, details) = match self {
            Self::Validation(error) => (error.error.clone(), Some(Json(error.details.clone()))),
            _ => ("".to_string(), None),
        };
        let body = ErrorBody {
            error: &message,
            details,
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PostError {
    #[error("validation error: {}. details: {}", .0.error, .0.details)]
    Validation(ValidationError),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Unexpected failure: {0}")]
    UnexpectedFailure(String),
}

impl ResponseError for PostError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::UnexpectedFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let (message, details) = match self {
            Self::Validation(error) => (error.error.clone(), Some(Json(error.details.clone()))),
            _ => ("".to_string(), None),
        };
        let body = ErrorBody {
            error: &message,
            details,
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}
