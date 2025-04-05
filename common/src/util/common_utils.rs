#![allow(unused_variables)]
#![allow(dead_code)]
use anyhow::Context;
use std::io::prelude::*;
use zip::{result::ZipError, write::SimpleFileOptions, CompressionMethod};

use hex::encode;
use std::fs::File;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::{DirEntry, WalkDir};


use crate::{AppError, AppState, UserCache};
use actix_web::{web, HttpRequest};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use uuid::Uuid;

pub fn copy_to<A, B>(a: A, b: B) -> B
where
    A: Serialize + for<'de> Deserialize<'de>,
    B: Serialize + for<'de> Deserialize<'de>,
{
    serde_json::from_value(serde_json::to_value(a).unwrap()).unwrap()
}


pub fn build_id() -> String {
    let uuid = Uuid::new_v4().simple();
    format!("{}", uuid)
}

pub fn build_snow_id() -> i64 {
    let mut generator = SafeSnowflake::new(1, 1);
   return generator.generate() as i64;
}
pub fn build_md5(content: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(content);
    let result = hasher.finalize();
    let hex_string = encode(result);
    hex_string
}



pub struct SafeSnowflake {
    node_id: u64,
    worker_id: u64,
    sequence: u64,
    last_timestamp: u64,
}

impl SafeSnowflake {
    pub fn new(node_id: u64, worker_id: u64) -> Self {
        Self {
            node_id: node_id & 0x1F,     // 5 bits
            worker_id: worker_id & 0x1F, // 5 bits
            sequence: 0,
            last_timestamp: 0,
        }
    }

    fn current_timestamp() -> u64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        now.as_millis() as u64 // 毫秒时间戳
    }

    pub fn generate(&mut self) -> u64 {
        let mut timestamp = Self::current_timestamp();

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & 0b11; // 2 bits
            if self.sequence == 0 {
                // 同一毫秒内超出最大序列，等下一毫秒
                while timestamp <= self.last_timestamp {
                    timestamp = Self::current_timestamp();
                }
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        // 拼接为 53 位以内的 ID:
        // 41 bits timestamp | 5 bits node_id | 5 bits worker_id | 2 bits sequence
        ((timestamp & 0x1FFFFFFFFFF) << 12) // 41 bits
            | ((self.node_id & 0x1F) << 7)  // 5 bits
            | ((self.worker_id & 0x1F) << 2) // 5 bits
            | (self.sequence & 0x03)         // 2 bits
    }
}



pub async fn  get_session_user(
    state: &web::Data<AppState>,
    req: HttpRequest,
) -> Result<UserCache, AppError> {
    if(1==1){
        return Ok(UserCache {
            id: 1,
            user_name: "admin".to_string(),
            is_admin: true,
            bucket_list: vec![],
        });
    }
    let mut token_value = "";
    let option = req.headers().get("Authorization");
    match option {
        Some(auth_value) => {
            if let Ok(auth_str) = auth_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token_key = &auth_str[9..];
                    let option = state.session_cache.get(token_key).await;
                    if let Some(user_cache) = option {
                        return Ok(user_cache.clone());
                    }
                }
            }
        }
        None => {
            return Err(AppError::NoRight("token.is.null".to_owned()));
        }
    }

    return Err(AppError::NoRight("token.is.null".to_owned()));
}


fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> anyhow::Result<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub fn do_zip_dir(src_dir: &Path, dst_file: &Path, method: zip::CompressionMethod) -> anyhow::Result<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}


pub fn zip_dir_to_tempfile(
    src_dir: &Path,
    method: CompressionMethod,
) -> Result<tempfile::TempDir,AppError> {
    // 创建临时目录
    let temp_dir = tempdir()?;

    // 目标 zip 文件路径
    let dst_path = temp_dir.path().join("default.zip");
    let file = File::create(&dst_path)?;

    // 遍历并压缩
    let walkdir = WalkDir::new(src_dir).into_iter();
    zip_dir(&mut walkdir.filter_map(|e| e.ok()), src_dir, file, method).map_err(|e|AppError::InternalError(e.to_string()));
    Ok(temp_dir)
}
