use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::Serialize;
use std::fmt::Debug;
use utoipa::{OpenApi, ToSchema};
// 统一返回vo
#[derive(Serialize, Debug, Clone)]
pub struct BaseResponse<T: Serialize + Debug> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}
#[derive(Debug)]
pub struct Metadata {
    pub name: String,
}

#[derive(Debug, MultipartForm,ToSchema)]
pub struct UploadForm {
    #[multipart(rename = "files",limit = "50mb")]
    #[schema(value_type = Vec<String>, format = Binary)]
    pub files: Vec<TempFile>,
    #[schema(value_type = String)]
    pub thumbnail_status:actix_multipart::form::text::Text<bool>,
    #[schema(value_type = String)]
    pub path:actix_multipart::form::text::Text<String>
}

impl BaseResponse<String> {
    pub fn ok_no_result() -> BaseResponse<String> {
        BaseResponse {
            msg: None,
            code: None,
            success: true,
            data: None,
        }
    }
    pub fn ok_result() -> BaseResponse<String> {
        BaseResponse {
            msg: None,
            code: None,
            success: true,
            data: Some("None".to_string()),
        }
    }
    pub fn ok_result_msg(msg: String) -> BaseResponse<String> {
        BaseResponse {
            msg: Option::from(msg),
            code: None,
            success: true,
            data: Some("None".to_string()),
        }
    }
    pub fn ok_result_code(code: String, msg: String) -> BaseResponse<String> {
        BaseResponse {
            msg: Option::from(msg),
            code: Option::from(code),
            success: true,
            data: Some("None".to_string()),
        }
    }

    pub fn err_result_code(code: String, msg: String) -> BaseResponse<String> {
        BaseResponse {
            msg: Option::from(msg),
            code: Option::from(code),
            success: false,
            data: Some("None".to_string()),
        }
    }
}

impl<T: Serialize + Debug> BaseResponse<T> {
    pub fn ok_result_data(data: T) -> BaseResponse<T> {
        BaseResponse {
            msg: None,
            code: None,
            success: true,
            data: Some(data),
        }
    }

    pub fn ok_result_none() -> BaseResponse<T> {
        BaseResponse {
            msg: None,
            code: None,
            success: true,
            data: None,
        }
    }

    pub fn err_result_msg(msg: String) -> BaseResponse<T> {
        BaseResponse {
            msg: Option::from(msg),
            code: Option::from("999".to_string()),
            success: false,
            data: None,
        }
    }
}

// 统一返回分页
#[derive(Serialize, Debug, Clone)]
pub struct ResponsePage<T: Serialize + Debug> {
    pub code: i32,
    pub msg: String,
    pub total: u64,
    pub success: bool,
    pub data: Option<T>,
}
impl<T: Serialize + Debug> ResponsePage<T> {
    pub fn ok_result_page(data: T, total: u64) -> ResponsePage<T> {
        ResponsePage {
            msg: "操作成功".to_string(),
            code: 0,
            success: true,
            data: Some(data),
            total,
        }
    }

    pub fn err_result_page(data: T, msg: String) -> ResponsePage<T> {
        ResponsePage {
            msg,
            code: 1,
            success: false,
            data: Some(data),
            total: 0,
        }
    }
}
