use crate::date_format::date_format;
use common::RightType;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;
use sqlx::types::Json;
use sqlx::{FromRow, MySqlPool, Type};
use std::path::Path;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};
//查询分页对像


#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub is_admin: bool,
    pub user_name: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub access_key: String,
    #[serde(skip_serializing)]
    pub secret_key: String,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
pub struct UserBucket {
    pub id: i64,
    pub user_id: i64,
    pub bucket_id: i64,
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

#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub id: i64,
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
    DIR
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
    WEBP,
    SVG,
    GIF,
    JPG,
    JPEG,
    PNG,
    TIFF,
    TIF,
    BMP,
    NONE,
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
            "webp" => ImageType::WEBP,
            "bmp" => ImageType::BMP,
            _ => ImageType::NONE,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, FromRow,Clone)]
pub struct FileInfo {
    pub id: i64,
    pub root: bool,
    pub bucket_id: i64,
    pub path_ref: i64,
    pub name: String,
    pub full_path: String,
    pub file_type: FileType,
    pub items: Json<Vec<FileItemDto>>,
    pub image_type: ImageType,
    pub size: u32,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct FileItemDto{
    pub path:String,
    pub size:u32
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, Default)]
pub struct PathDelTask{
    pub id: i64,
    pub path_id:i64,
    pub del_file_status:bool,
    pub del_path_status:bool
}
impl FileInfo {}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, Default)]
pub struct PathInfo {
    pub id: i64,
    pub bucket_id: i64,
    pub root: bool,
    pub path: String,
    pub parent: String,
    pub full_path: String,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
}

pub async fn get_conn(url: &String) -> MySqlPool {
    return MySqlPool::connect(&url)
        .await
        .expect("Failed to connect to database");
}
