use common::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool, Type};
use std::path::Path;
use std::str::FromStr;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter)]
pub enum RightType {
    PubRead,
    PubWrite,
    PubReadWrite,
    PriRead,
    PriWrite,
    PriReadWrite,
}

impl FromStr for RightType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pub_read" => Ok(RightType::PubRead),
            "pub_write" => Ok(RightType::PubWrite),
            "pub_read_write" => Ok(RightType::PubReadWrite),
            "pri_read" => Ok(RightType::PriRead),
            "pri_read_write" => Ok(RightType::PriReadWrite),
            _ => Err(()),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserInfo {
    pub id: String,
    pub user_name: String,
    pub password: String,
    pub access_key:String,
    pub secret_key:String
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserBucket {
    pub id: String,
    pub user_id: String,
    pub bucket_id: String,
    pub right_str: String,
}

impl UserBucket {
    fn get_rights(&self) -> Result<Vec<RightType>, AppError> {
        let mut vec = Vec::new();
        self.right_str.split(",").for_each(|r| {
            vec.push(RightType::PubReadWrite);
        });
        Ok(vec)
    }
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bucket {
    pub id: String,
    pub name: String,
    pub thumbnail_size: i32,
    pub quota: i32,
    pub current_quota: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
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
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FileInfo {
    pub id: String,
    pub root: bool,
    pub bucket_id: String,
    pub path_ref: String,
    pub name: String,
    pub file_type: FileType,
    pub items: String,
    pub image_type: ImageType,
    pub size: u32,
    pub thumbnail: String,
    pub thumbnail_size: i32,
    pub thumbnail_status: bool,
}

impl FileInfo {

}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct FileInfoTem {
    pub id: String,
    pub file_name: String,
    pub file_type: FileType,
    pub items: String,
    pub image_type: ImageType,
    pub size: i32,
    pub thumbnail: Option<String>,
    pub thumbnail_status: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone,Default)]
pub struct PathInfo {
    pub bucket_id: String,
    pub id: String,
    pub root: bool,
    pub path: String,
    pub parent: String,
    pub full_path: String,
}


pub async fn get_conn(url: &String) -> MySqlPool {
    return MySqlPool::connect(&url)
        .await
        .expect("Failed to connect to database");
}
