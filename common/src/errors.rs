use actix_multipart::MultipartError;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use sqlx::Error;
use sqlx::error::DatabaseError;
use thiserror::Error;
pub type Result<T> = std::result::Result<T, AppError>;
#[derive(Debug, Error)]
pub enum AppError {
    #[error("NotFound error: {0}")]
    NotFound(String),
    #[error("NotFound error: {0}")]
    NotErrorNoRight(String),
    #[error("Invalid Input: {0}")]
    InvalidInput(String),
    #[error("Internal Server Error {0}")]
    InternalError(String),
    #[error("Internal Server Error {0}")]
    RedisError(String),
    #[error("sqlx error: {0}")]
    DBError(#[from] Error),
    #[error("MultipartError Error: {0}")]
    MultipartError(#[from] MultipartError),
    #[error("io Error: {0}")]
    IoError(#[from] std::io::Error),
}

/// 将错误序列化为 JSON 响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let error_message = self.to_string();

        // 根据错误类型返回适当的 HTTP 状态码
        let status_code = match *self {
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::InvalidInput(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::IoError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::MultipartError(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::NotErrorNoRight(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::DBError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status_code).json(ErrorResponse {
            message: error_message,
        })
    }
}
