pub use crate::errors::*;
pub use crate::resp::*;
use crate::{AppState, FileType, ImageType, PathInfo};
use actix_multipart::Multipart;
use actix_multipart::form::MultipartForm;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_web::{HttpResponse, Responder, get, post, web};
use chrono::{Datelike, Timelike, Utc};
use futures_util::StreamExt;
use log::{debug, warn};
use moka::future::Cache;
use mysql::params;
use mysql::prelude::Queryable;
use serde::de::Unexpected::Option;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread::current;
use utoipa::ToSchema;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(upload);
}

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 每个文件分片大小（4MB）

#[utoipa::path(
    post,
    path = "/upload",
    responses(
        (status = 200, description = "文件上传成功"),
        (status = 400, description = "上传失败")
    ),
    request_body(content = UploadForm, content_type = "multipart/form-data")
)]
#[post("/upload")]
pub async fn upload(
    app_state: web::Data<AppState>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> std::result::Result<impl Responder, AppError> {
    if (form.files.len() == 0) {
        return Ok(web::Json(BaseResponse::err_result_code(
            "no.file".to_string(),
            "no file to upload".to_string(),
        )));
    }
    let mut conn = app_state.pool.get_conn()?;
    let mut current_path = form.path.to_string();
    let path_id: String;
    if (current_path.is_empty() || current_path == "/" || current_path == "") {
        current_path = String::from("");
    }
    path_id =
        check_and_save_path(&mut conn, &current_path.clone(), &app_state.db_path_cache).await?;
    let dir_name = build_dir_name(&app_state.root_path, &app_state.dir_create_cache).await?;

    for mut file in form.files {
        let mut uploaded_files = Vec::new();
        let file_size = file.size;
        let num_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
        for i in 0..num_chunks {
            let mut buffer = vec![0; CHUNK_SIZE];
            let bytes_read = match file.file.read(&mut buffer) {
                Ok(bytes) => bytes,
                Err(err) => {
                    eprintln!("read.file.buffer.error: {}", err);
                    return Ok(web::Json(BaseResponse::err_result_msg(
                        "io.error".to_string(),
                    )));
                }
            };

            let fid = Uuid::new_v4();
            // 定义分片文件的路径
            let chunk_file_path = format!("{}/{}", dir_name, fid.clone().to_string());

            // 创建并写入分片文件
            let mut chunk_file = match File::create(&chunk_file_path) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("create.file.error: {}", err);
                    return Ok(web::Json(BaseResponse::err_result_msg(
                        "io.error".to_string(),
                    )));
                }
            };
            uploaded_files.push(chunk_file_path);

            if let Err(err) = chunk_file.write_all(&buffer[..bytes_read]) {
                eprintln!("write.file.error: {}", err);
                return Ok(web::Json(BaseResponse::err_result_msg(
                    "io.error".to_string(),
                )));
            }
        }
        let fid = Uuid::new_v4();
        let file_name = file.file_name.as_ref().unwrap();
        let file_type = FileType::get_file_type(file_name);
        insert_file_name(
            &mut conn,
            fid.to_string(),
            &path_id,
            file_name,
            &file_type,
            uploaded_files,
            &file.size,
            &form.thumbnail_status,
        )?;
    }
    Ok(web::Json(BaseResponse::ok_no_result()))
}

///
///
/// 插入文件
fn insert_file_name(
    conn: &mut mysql::PooledConn,
    id: String,
    path_ref: &String,
    file_name: &String,
    file_type: &FileType,
    items: Vec<String>,
    size: &usize,
    thumbnail_status: &bool,
) -> mysql::error::Result<String, AppError> {
    let mut items_str = String::new();
    for item in items {
        items_str.push_str(&item);
        items_str.push_str(",");
    }
    let image_type = match file_type {
        FileType::IMAGE => ImageType::get_image_type(file_name),
        _ => ImageType::EMPTY,
    };
    let current_thumbnail_status;
    if (image_type != ImageType::EMPTY && *thumbnail_status) {
        current_thumbnail_status = true;
    } else {
        current_thumbnail_status = false;
    }
    conn.exec_drop(
        "
        insert into file_info
        (id,path_ref,file_name,file_type,image_type,items,size,thumbnail_status)VALUES
        (:id,:path_ref,:file_name,:file_type,:image_type,:items,:size,:thumbnail_status)
        ",
        params! {
            "id" => &id,
            "path_ref" => path_ref,
            "file_name" => file_name,
            "file_type" => file_type.to_string(),
            "image_type" => image_type.to_string(),
            "items" => items_str,
            "size" => size,
            "thumbnail_status" => current_thumbnail_status
        },
    )?;
    return Ok(id);
}

///
/// 检查目录是否存在
async fn check_and_save_path(
    conn: &mut mysql::PooledConn,
    full_path: &String,
    file_cache: &Arc<Cache<String, String>>,
) -> mysql::error::Result<String, AppError> {
    //判断缓存里是否存在文件夹
    let option = file_cache.get(full_path).await;
    let cache_dir_id = match option {
        Some(option) => option,
        None => "".to_string(),
    };
    if (!cache_dir_id.is_empty()) {
        return Ok(cache_dir_id);
    }
    let root: bool;
    if (full_path.eq("")) {
        root = true;
    } else {
        root = false;
    }
    let path_list = &full_path.split("/").collect::<Vec<&str>>();
    let mut current_dir: String = String::from("");
    let mut current_path_info;
    let mut parent_id: String = String::from("");

    let mut finally_id: String = String::new();
    for path_item in path_list.iter() {
        if (current_dir.is_empty()) {
            current_dir = format!("{}", path_item);
        } else {
            current_dir = format!("{}/{}", current_dir, &path_item);
        }
        let option = file_cache.get(&current_dir).await;
        let cache_dir_id = match option {
            Some(option) => option,
            None => "".to_string(),
        };
        if (cache_dir_id.is_empty()) {
            let path_list: Vec<PathInfo> = conn.exec_map(
                "SELECT id,root,path,parent,full_path FROM path_info where full_path=:full_path",
                params! {
                    "full_path" => &current_dir,
                },
                |(id, root, path, parent, full_path)| PathInfo {
                    id,
                    root,
                    path,
                    parent,
                    full_path,
                },
            )?;

            let mut has_db_dir = false;
            if path_list.len() == 1 {
                current_path_info = &path_list[0];
                has_db_dir = true;
                parent_id = current_path_info.clone().id;
                finally_id = parent_id.clone();
                //置入缓存
                file_cache.insert(parent_id.clone(), current_path_info.full_path.clone());
            }
            if (!has_db_dir) {
                let path_id = Uuid::new_v4().to_string();
                conn.exec_drop(
                    "
            INSERT INTO path_info ( id,root, path, parent,full_path)
            VALUES (:id, :root, :path, :parent, :full_path)
            ",
                    params! {
                        "id" => &path_id,
                        "root" => root,
                        "path" => &path_item,
                        "parent" => &parent_id,
                        "full_path" => &current_dir,
                    },
                )?;
                file_cache.insert(path_id.clone(), current_dir.clone());
                parent_id = path_id.clone();
                finally_id = path_id.clone();
            }
        } else {
            return Ok((finally_id));
        }
    }
    return Ok((finally_id));
}

/// **写入分片数据到文件**
fn write_chunk_to_file(filename: &str, data: &[u8]) -> std::io::Result<()> {
    let path = Path::new(filename);
    let mut file = OpenOptions::new().create(true).write(true).open(path)?;
    file.write_all(data)?;
    Ok(())
}

/// **防止路径遍历攻击**
fn sanitize_filename(filename: &str) -> String {
    filename.replace("/", "_").replace("\\", "_")
}

///生成文件目录
async fn build_dir_name(
    root: &String,
    cache: &Arc<Cache<String, String>>,
) -> std::result::Result<String, AppError> {
    let now = chrono::Local::now();
    //获取年
    let year = &now.year();
    let year_path_str = format!("{}/{}", root, year);
    if let Some(path) = cache.get(&year.to_string()).await {
    } else {
        let year_path = Path::new(&year_path_str);
        if (!year_path.exists()) {
            std::fs::create_dir_all(year_path);
        }
        cache.insert(year_path_str.clone(), "1".to_string()).await;
    }
    //获取一年中的第几天
    let day = &now.date_naive();
    let day_index = &day.ordinal(); // 获取一年中的第几天（1-366）
    let day_path_str = format!("{}/{}/{}", &root, year, day_index);
    if let Some(path) = cache.get(&day_path_str.to_string()).await {
    } else {
        let day_path = Path::new(&day_path_str);
        if (!day_path.exists()) {
            std::fs::create_dir_all(day_path);
        }
        cache.insert(day_path_str.clone(), "1".to_string()).await;
    }

    //获取一天中的第几分钟
    let minutes_of_day = &now.hour() * 60 + &now.minute();

    let minutes_path_str = format!("{}/{}/{}/{}", &root, &year, &day_index, minutes_of_day);
    if let Some(path) = cache.get(&minutes_path_str.to_string()).await {
    } else {
        let minutes_path = Path::new(&minutes_path_str);
        if (!minutes_path.exists()) {
            std::fs::create_dir_all(minutes_path);
        }
        cache
            .insert(minutes_path_str.clone(), "1".to_string())
            .await;
    }
    Ok(minutes_path_str)
}
