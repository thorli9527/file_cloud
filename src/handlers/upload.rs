pub use crate::errors::*;
pub use crate::resp::*;
use crate::{AppState, FileType, ImageType, PathInfo};
use actix_multipart::Multipart;
use actix_web::{post, web, Responder};
use chrono::{Datelike, Local, Timelike};
use futures::StreamExt;
use futures_util::AsyncReadExt;
use moka::future::Cache;
use sanitize_filename::sanitize;
use sqlx::MySqlPool;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    cfg.app_data(state.clone())
        .service(upload_file);
}

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // ✅ 4MB 分片

/// **处理文件上传（4MB 分片）**
#[post("/upload")]
async fn upload_file(
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> std::result::Result<impl Responder, AppError> {
    let mut path = String::new();
    let mut is_thumbnail: bool = true;
    let mut file_name: String = String::new();
    let mut uploaded_files = Vec::new();
    let mut size: usize = 0;
    while let Some(field) = payload.next().await {
        let mut field = field.unwrap();

        let content_disposition = field.content_disposition().unwrap();
        let dir_name = build_dir_name(&app_state.root_path, &app_state.dir_create_cache).await?;

        match content_disposition.get_name().unwrap() {
            "path" => {
                // 读取普通表单字段（文本）
                let mut data = String::new();
                while let Some(chunk) = field.next().await {
                    data.push_str(&String::from_utf8_lossy(&chunk?));
                }
                path = data;
            }
            "is_thumbnail" => {
                // 读取普通表单字段（文本）
                let mut data = String::new();
                while let Some(chunk) = field.next().await {
                    data.push_str(&String::from_utf8_lossy(&chunk?));
                }
                is_thumbnail = match data.to_lowercase().as_str() {
                    // "true" | "1" | "on" | "yes" => true,
                    "false" | "0" | "off" | "no" => false,
                    _ => true,
                };
            }
            "file" => {
                let mut fid = Uuid::new_v4();
                // 定义分片文件的路径
                let mut buffer = Vec::with_capacity(CHUNK_SIZE);
                file_name = field
                    .content_disposition()
                    .unwrap()
                    .get_filename()
                    .unwrap()
                    .to_string();
                if (file_name.is_empty()) {
                    return Ok(web::Json(BaseResponse::err_result_msg(
                        "Invalid file_name value",
                    )));
                }
                while let Some(Ok(bytes)) = field.next().await {
                    buffer.extend_from_slice(&bytes); // ✅ 累积数据
                    size += bytes.len();
                    if buffer.len() >= CHUNK_SIZE {
                        //写入文件
                        let chunk_path = format!("{}/{}", dir_name, fid.clone().to_string());
                        uploaded_files.push(chunk_path.clone());
                        let mut file = File::create(&chunk_path).unwrap();
                        file.write_all(&buffer[..CHUNK_SIZE]).unwrap();

                        buffer.drain(..CHUNK_SIZE); // ✅ 清除已写入的部分
                        fid = Uuid::new_v4();
                    }
                }

                // 处理剩余数据（小于 4MB）
                if !buffer.is_empty() {
                    // let chunk_name = format!("{}/{}_chunk_{}", UPLOAD_DIR, file_id, chunk_index);
                    // let mut file = File::create(&chunk_name).unwrap();
                    // file.write_all(&buffer).unwrap();
                    //
                    let chunk_path = format!("{}/{}", dir_name, fid.clone().to_string());
                    uploaded_files.push(chunk_path.clone());
                    let mut file = File::create(&chunk_path).unwrap();
                    file.write_all(&buffer).unwrap();
                }
            }
            _ => {} // 忽略未知字段
        }
    }
    let fid = Uuid::new_v4();

    let file_type = FileType::get_file_type(&file_name);
    let mut thumbnail_status_tem: &bool;
    if (file_type != FileType::IMAGE) {
        thumbnail_status_tem = &true;
    } else {
        thumbnail_status_tem = &is_thumbnail;
    }
    if (path.len() > 128) {
        return Ok(web::Json(BaseResponse::err_result_msg(
            "path name to lang (max=128)",
        )));
    }
    let mut current_path = path;
    let path_id: String;
    if (current_path.is_empty() || current_path == "/" || current_path == "") {
        current_path = String::from("");
        path_id = current_path.clone();
    } else {
        path_id = check_and_save_path(
            &app_state.pool,
            &current_path.clone(),
            &app_state.db_path_cache,
        )
        .await?;
    }
    if (file_name.len() > 64) {
        return Ok(web::Json(BaseResponse::err_result_msg(
            "file name to lang (max=64)",
        )));
    }
    insert_file_name(
        &app_state.pool,
        fid.clone().to_string(),
        &path_id,
        &file_name,
        &file_type,
        uploaded_files,
        &size,
        thumbnail_status_tem,
    )
    .await?;
    Ok(web::Json(BaseResponse::ok_result_data(fid.to_string())))
}

async fn insert_file_error(conn: &MySqlPool, error_files: Vec<String>) -> Result<()> {
    let mut items_str = String::new();
    for item in error_files {
        items_str.push_str(&item);
        items_str.push_str(",");
    }

    sqlx::query(
        "insert into file_info (file, error_count, del_status, create_time)VALUES(?,0,:0,?)",
    )
    .bind(&items_str)
    .bind(Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
    .execute(conn)
    .await
    .unwrap();

    Ok(())
}

///
///
/// 插入文件
async fn insert_file_name(
    conn: &MySqlPool,
    id: String,
    path_ref: &String,
    file_name: &String,
    file_type: &FileType,
    items: Vec<String>,
    size: &usize,
    thumbnail_status: &bool,
) -> Result<()> {
    let mut items_str = String::new();
    for item in items {
        items_str.push_str(&item);
        items_str.push_str(",");
    }
    let image_type = match &file_type {
        FileType::IMAGE => ImageType::get_image_type(file_name),
        _ => ImageType::EMPTY,
    };
    let mut current_thumbnail_status;

    if (image_type != ImageType::EMPTY && *thumbnail_status) {
        current_thumbnail_status = false;
    } else {
        current_thumbnail_status = match image_type {
            ImageType::JPEG => true,
            ImageType::BMP => true,
            ImageType::JPG => true,
            ImageType::PNG => true,
            ImageType::TIFF => true,
            ImageType::TIF => true,
            ImageType::WebP => true,
            _ => false,
        };
        //如果图片小于 100K,则不进行压缩
        if size > &(CHUNK_SIZE / 10 / 4) {
            current_thumbnail_status = false;
        }
    }

    let root: bool;
    if (path_ref.eq("")) {
        root = true;
    } else {
        root = false;
    }

    let i = u32::try_from(*size).unwrap();
    sqlx::query(
        "INSERT INTO file_info (id,path_ref,file_name,file_type,image_type,items,size,thumbnail_status,root)VALUES(?,?,?,?,?,?,?,?,?)")
        .bind(&id)
        .bind(path_ref)
        .bind(file_name)
        .bind(&file_type)
        .bind(&image_type)
        .bind(items_str)
        .bind(i)
        .bind(current_thumbnail_status)
        .bind(root)
        .execute(conn)
        .await.unwrap();
    return Ok(());
}

///
/// 检查目录是否存在
async fn check_and_save_path(
    conn: &MySqlPool,
    full_path: &String,
    db_path_cache: &Arc<Cache<String, String>>,
) -> Result<String> {
    let safe_path = sanitize(&full_path);
    //判断缓存里是否存在文件夹
    let option = db_path_cache.get(&safe_path.to_string()).await;
    let cache_dir_id = match option {
        Some(option) => option,
        None => "".to_string(),
    };
    if (!cache_dir_id.is_empty()) {
        return Ok(cache_dir_id);
    }
    let root: bool;
    if (safe_path.eq("")) {
        root = true;
    } else {
        root = false;
    }

    let path_list = &safe_path.split("/").collect::<Vec<&str>>();
    let mut current_dir: String = String::from("");
    let mut current_path_info;
    let mut parent_id: String = String::from("");

    let mut finally_id: String = String::new();
    for path_item in path_list.iter() {
        if (current_dir.is_empty()) {
            current_dir = format!("{}", path_item);
        } else {
            current_dir = format!("{}/{}", current_dir.clone(), &path_item);
        }
        let option = db_path_cache.get(current_dir.clone().as_str()).await;
        let cache_dir_id = match option {
            Some(option) => option,
            None => "".to_string(),
        };
        if (cache_dir_id.is_empty()) {
            let list_db = sqlx::query_as::<_, PathInfo>(
                "SELECT id,root,path,parent,full_path FROM path_info where full_path= ?",
            )
            .bind(current_dir.clone()) // ✅ 绑定参数
            .fetch_all(conn)
            .await
            .unwrap();
            let mut has_db_dir = false;
            if list_db.len() == 1 {
                current_path_info = &list_db[0];
                has_db_dir = true;
                parent_id = current_path_info.clone().id;
                finally_id = parent_id.clone();
                //置入缓存
                &db_path_cache
                    .insert(current_dir.clone(), finally_id.clone())
                    .await;
            }
            if (!has_db_dir) {
                let path_id = Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO path_info ( id,root, path, parent,full_path) VALUES (?, ?, ?, ?, ?)"
                )
                    .bind(&path_id)
                    .bind(root)
                    .bind(&path_item)
                    .bind(&parent_id)
                    .bind(&current_dir)
                    .execute(conn)
                    .await.unwrap();
                db_path_cache
                    .insert(current_dir.clone(), path_id.clone())
                    .await;
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
    if let None = cache.get(&year_path_str.to_string()).await {
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
    if let None = cache.get(&day_path_str.to_string()).await {
        let day_path = Path::new(&day_path_str);
        if (!day_path.exists()) {
            std::fs::create_dir_all(day_path);
        }
        cache.insert(day_path_str.clone(), "1".to_string()).await;
    }

    //获取一天中的第几分钟
    let minutes_of_day = &now.hour() * 60 + &now.minute();

    let minutes_path_str = format!("{}/{}/{}/{}", &root, &year, &day_index, minutes_of_day);
    if let None = cache.get(&minutes_path_str.to_string()).await {
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
