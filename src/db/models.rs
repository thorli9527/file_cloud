use mysql::prelude::*;
use mysql::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathInfo {
    pub id: String,
    pub root: bool,
    pub path: String,
    pub parent: String,
    pub full_path: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct FileItem {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone,PartialEq, Eq)]
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
            "jpg" | "jpeg" | "png" | "gif" | "bmp" |"tif"| "tiff" | "webp" => FileType::IMAGE,
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

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FileType::SVG => write!(f, "svg"),
            FileType::TEXT => write!(f, "text"),
            FileType::SCRIPT => write!(f, "script"),
            FileType::DOC => write!(f, "doc"),
            FileType::EXCEL => write!(f, "excel"),
            FileType::NORMAL => write!(f, "normal"),
            FileType::IMAGE => write!(f, "image"),
            FileType::VIDEO => write!(f, "video"),
            FileType::AUDIO => write!(f, "audio"),
            FileType::ZIP => write!(f, "zip"),
        }
    }
}

#[derive(Debug, Deserialize, Clone,PartialEq, Eq)]
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
    EMPTY
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
impl Display for ImageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ImageType::WebP => write!(f, "webp"),
            ImageType::GIF => write!(f, "gif"),
            ImageType::JPG => write!(f, "jpg"),
            ImageType::JPEG => write!(f, "jpeg"),
            ImageType::PNG => write!(f, "png"),
            ImageType::TIFF => write!(f, "tiff"),
            ImageType::TIF => write!(f, "tif"),
            ImageType::BMP => write!(f, "bmp"),
            ImageType::SVG => write!(f, "svg"),
            ImageType::EMPTY => write!(f, "empty"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileInfo {
    pub id: String,
    pub path_ref: String,
    pub name: String,
    pub file_type: FileType,
    pub items: Vec<FileItem>,
    pub image_type: ImageType,
    pub thumbnail: String,
    pub thumbnail_status: bool,
}

pub fn get_conn(url: String) -> mysql::Pool {
    Pool::new(url.as_str()).unwrap()
}
