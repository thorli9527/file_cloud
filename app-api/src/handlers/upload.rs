use actix_multipart::Multipart;
use actix_web::{HttpRequest, Responder, post, web};
use chrono::{Datelike, Local, Timelike};
use common::{AppError, AppState, BaseResponse, build_id};
use futures_util::StreamExt;
use log::info;
use model::*;
use moka::future::Cache;
use sanitize_filename::sanitize;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
use web::Data;
pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone()).service(upload_file);
}

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // ✅ 4MB 分片

/// **处理文件上传（4MB 分片）**
#[post("/upload/{bucket}")]
pub async fn upload_file(
    bucket: web::Path<String>,
    app_state: Data<AppState>,
    path_info_rep: Data<PathRepository>,
    bucket_rep: Data<BucketRepository>,
    user_bucket_right_rep: Data<UserBucketRightRepository>,
    file_rep: Data<FileRepository>,
    req: HttpRequest,
    mut payload: Multipart,
) -> std::result::Result<impl Responder, AppError> {
    if (bucket.is_empty()) {
        return Err(AppError::NotErrorNoRight("bucket.error".to_owned()));
    }
    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("name", bucket.to_string());
    let bucket_result = bucket_rep.dao.find_by_one(params).await;
    let bucket_info = match bucket_result {
        Ok(info) => info,
        Err(e) => return Err(AppError::NotErrorNoRight(e.to_string())),
    };
    if (!bucket_info.pub_write) {
        let secret_key_header = req.headers().get("secret_key");
        let secret_key = match secret_key_header {
            Some(secret_key) => secret_key.to_str().unwrap().to_string(),
            None => {
                return Err(AppError::NotErrorNoRight("no.right".to_string()));
            }
        };

        let access_key = req.headers().get("Access-Key");
        match access_key {
            Some(key) => {
                params = HashMap::new();
                let access_key_value = key.to_str().unwrap().to_string();
                params.insert("access_key", access_key_value);
                params.insert("bucket_name", bucket.to_string());
                let user_bucket_right_result =
                    user_bucket_right_rep.dao.query_by_params(params).await?;
                let mut has_right: bool = false;
                for bucket_right in user_bucket_right_result {
                    if bucket_right.right == RightType::Write {
                        has_right = true;
                        if secret_key == bucket_right.secret_key {
                            return Err(AppError::NotErrorNoRight("no.right".to_string()));
                        }
                    }
                }
                if !&has_right {
                    return Err(AppError::NotErrorNoRight("no.right".to_string()));
                }
            }
            None => {
                return Err(AppError::NotErrorNoRight("no.right".to_string()));
            }
        }
    }

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
                let mut fid = build_id();
                // 定义分片文件的路径
                let mut buffer = Vec::with_capacity(CHUNK_SIZE);
                file_name = field
                    .content_disposition()
                    .unwrap()
                    .get_filename()
                    .unwrap()
                    .to_string();
                if file_name.is_empty() {
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
                        fid = build_id();
                    }
                }
                // 处理剩余数据（小于 4MB）
                if !buffer.is_empty() {
                    let chunk_path = format!("{}/{}", dir_name, fid.clone().to_string());
                    uploaded_files.push(chunk_path.clone());
                    let mut file = File::create(&chunk_path).unwrap();
                    file.write_all(&buffer).unwrap();
                }
            }
            _ => {} // 忽略未知字段
        }
    }
    let fid = build_id();
    //s
    let file_type = FileType::get_file_type(&file_name);
    let mut thumbnail_status_tem: &bool;
    if file_type != FileType::IMAGE {
        thumbnail_status_tem = &true;
    } else {
        thumbnail_status_tem = &is_thumbnail;
    }
    if path.len() > 128 {
        return Ok(web::Json(BaseResponse::err_result_msg(
            "path name to lang (max=128)",
        )));
    }
    let mut current_path = path;
    let path_id: String;
    if current_path.is_empty() || current_path == "/" || current_path == "" {
        current_path = String::from("");
        path_id = current_path.clone();
    } else {
        path_id = check_and_save_path(
            &current_path.clone(),
            &app_state.db_path_cache,
            &path_info_rep,
        )
        .await?;
    }
    if file_name.len() > 64 {
        return Ok(web::Json(BaseResponse::err_result_msg(
            "file name to lang (max=64)",
        )));
    }
    insert_file_name(
        &file_rep,
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

async fn insert_file_error(conn: &MySqlPool, error_files: Vec<String>) -> Result<(), AppError> {
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
    file_rep: &FileRepository,
    id: String,
    path_ref: &String,
    file_name: &String,
    file_type: &FileType,
    items: Vec<String>,
    size: &usize,
    thumbnail_status: &bool,
) -> Result<(), AppError> {
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

    if image_type != ImageType::EMPTY && *thumbnail_status {
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
    if path_ref.eq("") {
        root = true;
    } else {
        root = false;
    }

    let i = u32::try_from(*size).unwrap();
    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("id", id.clone());
    params.insert("path_ref", path_ref.clone());
    params.insert("file_name", file_name.clone());
    params.insert("file_type", file_type.as_ref().to_string());
    params.insert("image_type", image_type.as_ref().to_string());
    params.insert("thumbnail_status", thumbnail_status.to_string());
    params.insert("thumbnail", thumbnail_status.to_string());
    params.insert("items", items_str);
    params.insert(
        "root",
        match root {
            true => "1".to_string(),
            false => "0".to_string(),
        },
    );
    params.insert("size", size.to_string());
    file_rep.dao.insert(params);
    Ok(())
}

///
/// 检查目录是否存在
async fn check_and_save_path(
    full_path: &String,
    db_path_cache: &Arc<Cache<String, String>>,
    path_info_rep: &PathRepository,
) -> Result<String, AppError> {
    let safe_path = sanitize(&full_path);
    //判断缓存里是否存在文件夹
    let option = db_path_cache.get(&safe_path.to_string()).await;
    let cache_dir_id = match option {
        Some(option) => option,
        None => "".to_string(),
    };
    if !cache_dir_id.is_empty() {
        return Ok(cache_dir_id);
    }
    let root: bool;
    if safe_path.eq("") {
        root = true;
    } else {
        root = false;
    }

    let path_list = &safe_path.split("/").collect::<Vec<&str>>();
    let mut current_dir: String = String::from("");
    let mut parent_id: String = String::from("");

    let mut finally_id: String = String::new();
    for path_item in path_list.iter() {
        if current_dir.is_empty() {
            current_dir = format!("{}", path_item);
        } else {
            current_dir = format!("{}/{}", current_dir.clone(), &path_item);
        }
        let option = db_path_cache.get(current_dir.clone().as_str()).await;
        let cache_dir_id = match option {
            Some(option) => option,
            None => "".to_string(),
        };
        if cache_dir_id.is_empty() {
            let mut path_info = PathInfo::default();
            path_info.full_path = current_dir.clone();
            let mut params: HashMap<&str, String> = HashMap::new();
            params.insert("full_path", current_dir.to_string());
            let result = path_info_rep.dao.find_by_one(params).await;
            match result {
                Ok(info) => {
                    path_info = info;
                    finally_id = path_info.id;
                }
                Err(e) => {
                    // write!("not.found:{:}", "{}", *path_item);
                    params = HashMap::new();
                    let current_id = build_id();
                    params.insert("id", current_id.clone());
                    params.insert(
                        "root",
                        match root {
                            true => "1".to_string(),
                            false => "0".to_string(),
                        },
                    );
                    params.insert("path", path_item.to_string());
                    params.insert("parent", parent_id.to_string());
                    params.insert("full_path", full_path.to_string());
                    path_info_rep.dao.insert(params).await?;
                }
            }
        } else {
            return Ok(finally_id);
        }
    }
    return Ok(finally_id);
}

/// **防止路径遍历攻击**
fn sanitize_filename(filename: &str) -> String {
    filename.replace("/", "_").replace("\\", "_")
}
//
// ///生成文件目录
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
        if !year_path.exists() {
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
        if !day_path.exists() {
            std::fs::create_dir_all(day_path);
        }
        cache.insert(day_path_str.clone(), "1".to_string()).await;
    }

    //获取一天中的第几分钟
    let minutes_of_day = &now.hour() * 60 + &now.minute();

    let minutes_path_str = format!("{}/{}/{}/{}", &root, &year, &day_index, minutes_of_day);
    if let None = cache.get(&minutes_path_str.to_string()).await {
        let minutes_path = Path::new(&minutes_path_str);
        if !minutes_path.exists() {
            std::fs::create_dir_all(minutes_path);
        }
        cache
            .insert(minutes_path_str.clone(), "1".to_string())
            .await;
    }
    Ok(minutes_path_str)
}
