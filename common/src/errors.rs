use actix_multipart::form::json;
use actix_multipart::MultipartError;
use actix_web::{http, HttpResponse, Responder, ResponseError};
use actix_web::http::StatusCode;
use futures_util::future::err;
use log::{error, warn};
use redis::RedisError;
use serde::Serialize;
use serde_json::{json, to_string};
use sqlx::Error;
use thiserror::Error;
pub type Result<T> = std::result::Result<T, AppError>;
#[derive(Debug, Error)]
pub enum AppError {
    #[error("NotFound error: {0}")]
    NotFound(String),
    #[error("{0}")]
    BizError(String),
    #[error("NotFound error: {0}")]
    NoRight(String),
    #[error("Invalid Input: {0}")]
    InvalidInput(String),
    #[error("Internal Server Error {0}")]
    InternalError(String),
    #[error("Redis Error {0}")]
    RedisError(#[from] RedisError),
    #[error("sqlx error: {0}")]
    DBError(#[from] Error),
    #[error("MultipartError Error: {0}")]
    MultipartError(#[from] MultipartError),
    #[error("io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("validator error: {0}")]
    ValidateError(#[from] validator::ValidationError),
}

/// 将错误序列化为 JSON 响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse<'a> {
    status: u16,
    success:bool,
    msg: &'a str,
    errors: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // 根据错误类型返回适当的 HTTP 状态码
        let (code,success, error_type, error_msg) = match *&self {
            AppError::NotFound(msg) => {
                error!("InvalidInput: {}", msg);
                (actix_web::http::StatusCode::NOT_FOUND,false, "Not Found", "".to_string())
            },
            AppError::InvalidInput(msg) =>{
                error!("InvalidInput: {}", msg);
                (actix_web::http::StatusCode::BAD_REQUEST,true, "Invalid", "".to_string())
            },
            AppError::InternalError(msg) => {
                error!("IoError: {}", msg);
                (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,false, "Internal Error", "".to_string())
            },
            AppError::RedisError(msg) => {
                error!("IoError: {}", msg.to_string());
                (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,false, "Internal Redis Error", "".to_string())
            },
            AppError::IoError(msg) => {
                error!("IoError: {}", msg);
                (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,false, "Internal Error", "".to_string())
            },
            AppError::MultipartError(msg) => {
                error!("MultipartError Error: {}", msg);
                (actix_web::http::StatusCode::OK,true, "MultipartError Error", "".to_string())
            },
            AppError::NoRight(msg) => {
                warn!("NoRight Error: {}", msg);
                (actix_web::http::StatusCode::OK, true,msg.as_str(), "".to_string())
            },
            AppError::DBError(msg) => {
                error!("DB Error: {}", msg);
                (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,false, "Internal Error", "".to_string())
            },
            AppError::BizError(msg) => {
                warn!("Biz Error: {}", msg);
                (actix_web::http::StatusCode::OK,true, msg.as_str(), "".to_string())
            },
            AppError::ValidateError(err) => {
                let msg = err.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| "ValidateError".to_string());
                let json = serde_json::json!({ err.code.as_ref(): [msg] });
                let string = to_string(&json).unwrap();
                warn!("ValidateError Error: {}", &string);
                { (actix_web::http::StatusCode::OK, true,"ValidateError", string) }
            },
        };

        let err = ErrorResponse {
            status: code.as_u16(),
            msg: error_type,
            success,
            errors: error_msg,
        };

        HttpResponse::build(code)
            .content_type("application/json")
            .json(err)
    }
}
