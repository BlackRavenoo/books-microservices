
use actix_web::HttpResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Service configuration error")]
    ConfigError,
    
    #[error("Backend service error: {0}")]
    ServiceError(String),
    
    #[error("Invalid request parameters")]
    ValidationError,
    
    #[error("Resource not found")]
    NotFound,
}

impl actix_web::error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::ConfigError => HttpResponse::InternalServerError().json("Configuration error"),
            Self::ServiceError(msg) => HttpResponse::BadGateway().json(msg),
            Self::ValidationError => HttpResponse::BadRequest().json("Invalid parameters"),
            Self::NotFound => HttpResponse::NotFound().finish(),
        }
    }
}