use common::AppError;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Local, NaiveDateTime, Utc};
use sqlx::{FromRow, MySqlPool, Type};
use std::any::Any;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use async_trait::async_trait;
use sqlx::mysql::MySqlRow;
use strum_macros::{AsRefStr, EnumIter, EnumString, ToString};
use utoipa::ToSchema;
use crate::date_format::date_format;

//查询分页对像

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type, EnumString, ToSchema)]
#[sqlx(type_name = "ENUM")] // **告诉 `sqlx` 这是 `ENUM` 类型**
#[sqlx(rename_all = "lowercase")]
pub enum RightType {
    Read,
    Write,
    ReadWrite
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema,Clone)]
pub struct UserInfo {
    pub id: String,
    pub user_name: String,
    pub password: String,
    pub access_key: String,
    pub secret_key: String,
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
pub struct UserBucket {
    pub id: String,
    pub user_id: String,
    pub bucket_id: String,
    pub right: RightType,
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
pub struct UserBucketRight {
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub right: RightType,
}

#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
pub struct UserBucketRightQueryResult {
    pub bucket_name: String,
    pub right: RightType,
}

#[derive(Debug, Serialize, Deserialize, FromRow,ToSchema,Clone)]
pub struct Bucket {
    pub id: String,
    pub name: String,
    pub quota: i32,
    pub current_quota: i32,
    pub pub_read: bool,
    pub pub_write: bool,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type, EnumString, AsRefStr)]
#[sqlx(type_name = "ENUM")] // **告诉 `sqlx` 这是 `ENUM` 类型**
#[sqlx(rename_all = "lowercase")]
pub enum FileType {
    IMAGE,
    SVG,
    NORMAL,
    TEXT,
    DOC,
    SCRIPT,
    ZIP,
    VIDEO,
    AUDIO,
    EXCEL,
}
impl FileType {
    pub fn get_file_type(file_path: &str) -> FileType {
        let ext = Path::new(file_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tif" | "tiff" | "webp" => FileType::IMAGE,
            "svg" | "ai" | "eps" => FileType::SVG,
            "mp4" | "mkv" | "avi" | "mov" | "flv" => FileType::VIDEO,
            "mp3" | "wav" | "flac" | "aac" => FileType::AUDIO,
            "txt" | "md" | "json" | "xml" | "toml" | "conf" => FileType::TEXT,
            "zip" | "rar" | "tar" | "gz" => FileType::ZIP,
            "c" | "cpp" | "py" | "js" | "html" | "css" | "java" | "rs" | "go" | "cs" => {
                FileType::SCRIPT
            }
            "doc" | "docx" | "odt" | "rtf" | "pdf" => FileType::DOC,
            "xls" | "xlsx" | "ods" | "csv" | "tsv" => FileType::EXCEL,
            _ => FileType::NORMAL,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type, EnumString, AsRefStr)]
#[sqlx(type_name = "ENUM")] // **告诉 `sqlx` 这是 `ENUM` 类型**
#[sqlx(rename_all = "lowercase")]
pub enum ImageType {
    WebP,
    SVG,
    GIF,
    JPG,
    JPEG,
    PNG,
    TIFF,
    TIF,
    BMP,
    EMPTY,
}

impl ImageType {
    pub fn get_image_type(file_path: &str) -> ImageType {
        let ext = Path::new(file_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "jpg" => ImageType::JPG,
            "jpeg" => ImageType::JPEG,
            "svg" => ImageType::SVG,
            "png" => ImageType::PNG,
            "gif" => ImageType::GIF,
            "tiff" => ImageType::TIFF,
            "tif" => ImageType::TIF,
            "webp" => ImageType::WebP,
            "bmp" => ImageType::BMP,
            _ => ImageType::EMPTY,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
pub struct FileInfo {
    pub id: String,
    pub root: bool,
    pub bucket_id: String,
    pub path_ref: String,
    pub file_name: String,
    pub file_type: FileType,
    pub items: String,
    pub image_type: ImageType,
    pub size: u32,
    pub thumbnail: String,
    pub thumbnail_size: i32,
    pub thumbnail_status: bool,
    pub create_time: i64,
}

impl FileInfo {}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, Default)]
pub struct PathInfo {
    pub bucket_id: String,
    pub id: String,
    pub root: bool,
    pub path: String,
    pub parent: String,
    pub full_path: String,
    pub create_time: i64,
}

pub async fn get_conn(url: &String) -> MySqlPool {
    return MySqlPool::connect(&url)
        .await
        .expect("Failed to connect to database");
}
