use actix_multipart::Multipart;
use actix_web::{post, web, App, HttpRequest, Responder};
use chrono::{Datelike, Local, Timelike};
use common::{build_id, build_snow_id, build_time, get_session_user, result, result_data, result_error_msg, AppError, AppState, RightType};
use futures_util::StreamExt;
use model::*;
use moka::future::Cache;
use sqlx::{FromRow, MySqlPool};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use web::Data;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone()).service(upload_file);
}

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // ✅ 4MB 分片


/// **处理文件上传（4MB 分片）**
#[post("/upload/{bucket_id}/{pathId}")]
pub async fn upload_file(
    parmas: web::Path<(i64, i64)>,
    app_state: web::Data<AppState>,
    path_info_rep: web::Data<PathRepository>,
    user_bucket_rep: web::Data<UserBucketRepository>,
    file_rep: web::Data<FileRepository>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<impl Responder, AppError> {
    let (bucket_id, path_id) = parmas.into_inner();

    if (path_id != 0) {
        let path_info = path_info_rep.dao.find_by_id(path_id).await?;
        if (path_info.bucket_id != bucket_id) {
            return Err(AppError::InvalidInput("InvalidInput.params".to_owned()));
        }
    }
    let user_cache = get_session_user(&app_state, req).await?;
    let mut right = false;
    if (user_cache.is_admin) {
        right = true;
    }
    if (right == false) {
        let mut user_bucket_list_right = user_bucket_rep.query_by_user_id_and_bucket_Id(&user_cache.id, &&bucket_id).await?;
        if (user_bucket_list_right.is_empty()) {
            return Ok(web::Json(result_error_msg("no.right")));
        }
    }

    let mut path = String::new();
    let mut file_name: String = String::new();
    let mut uploaded_files: Vec<FileItemDto> = Vec::new();
    let mut size: usize = 0;
    while let Some(field) = payload.next().await {
        let mut field = field.unwrap();

        let content_disposition = field.content_disposition().unwrap();
        let dir_name = build_dir_name(&app_state.root_path, &app_state.dir_create_cache).await?;

        match content_disposition.get_name().unwrap() {
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
                    return Ok(web::Json(result_error_msg("Invalid file_name value")));
                }
                while let Some(Ok(bytes)) = field.next().await {
                    buffer.extend_from_slice(&bytes); // ✅ 累积数据
                    size += bytes.len();
                    if buffer.len() >= CHUNK_SIZE {
                        //写入文件
                        let chunk_path = format!("{}/{}", dir_name, fid.clone().to_string());
                        let file_item = FileItemDto {
                            path: chunk_path.clone(),
                            size: buffer.len() as u32,
                        };
                        uploaded_files.push(file_item);
                        let mut file = File::create(&chunk_path).unwrap();
                        file.write_all(&buffer[..CHUNK_SIZE]).unwrap();

                        buffer.drain(..CHUNK_SIZE); // ✅ 清除已写入的部分
                        fid = build_id();
                    }
                }
                // 处理剩余数据（小于 4MB）
                if !buffer.is_empty() {
                    let chunk_path = format!("{}/{}", dir_name, fid.clone().to_string());
                    let file_item = FileItemDto {
                        path: chunk_path.clone(),
                        size: buffer.len() as u32,
                    };
                    uploaded_files.push(file_item);
                    let mut file = File::create(&chunk_path).unwrap();
                    file.write_all(&buffer).unwrap();
                }
            }
            _ => {} // 忽略未知字段
        }
    }

    let fid = build_snow_id();
    //s
    let file_type = FileType::get_file_type(&file_name);

    if path_id != 0
    {
        path = path_info_rep.dao.find_by_id(path_id).await?.full_path;
    }

    if file_name.len() > 64 {
        return Ok(web::Json(result_error_msg("file name to lang (max=64)")));
    }
    insert_file_name(&bucket_id, &file_rep, fid, path_id, &file_name, &path, &file_type, uploaded_files, &size).await?;
    // Ok(web::Json(result_data(fid.to_string())))
    Ok(web::Json(result()))
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
    bucket_id: &i64,
    file_rep: &FileRepository,
    id: i64,
    path_ref: i64,
    name: &str,
    full_path: &String,
    file_type: &FileType,
    items: Vec<FileItemDto>,
    size: &usize,
) -> Result<(), AppError> {
    let image_type = match &file_type {
        FileType::IMAGE => ImageType::get_image_type(name),
        _ => ImageType::NONE,
    };

    let root: bool;
    if path_ref == 0 {
        root = true;
    } else {
        root = false;
    }

    let i = u32::try_from(*size).unwrap();
    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("id", id.to_string());
    params.insert("bucket_id", bucket_id.to_string());
    params.insert("path_ref", path_ref.to_string());
    params.insert("name", name.to_string());
    if (root) {
        params.insert("full_path", full_path.to_owned());
    } else {
        params.insert("full_path", format!("{}/", full_path));
    }

    params.insert("file_type", file_type.as_ref().to_string());
    params.insert("image_type", image_type.as_ref().to_string());
    params.insert(
        "root",
        match root {
            true => "1".to_string(),
            false => "0".to_string(),
        },
    );
    params.insert("size", size.to_string());
    let now = Local::now();
    params.insert("create_time", now.format("%Y-%m-%d %H:%M:%S").to_string());
    file_rep.insert(params, &items).await?;
    Ok(())
}


///
/// 检查目录是否存在
async fn check_and_save_path(
    bucket_id: &i64,
    full_path: &String,
    db_path_cache: &Arc<Cache<String, String>>,
    path_info_rep: &PathRepository,
) -> Result<i64, AppError> {
    //判断缓存里是否存在文件夹
    let option = db_path_cache.get(&full_path.to_string()).await;
    let cache_dir_id = match option {
        Some(option) => option,
        None => "".to_string(),
    };
    if !cache_dir_id.is_empty() {
        return Ok(cache_dir_id.parse().unwrap());
    }
    let mut root: bool;
    if full_path.eq("") {
        root = true;
    } else {
        root = false;
    }

    let path_list = &full_path.split("/").collect::<Vec<&str>>();
    let mut current_dir: String = String::from("");
    let mut parent_id: String = String::from("");

    let mut finally_id = 0;
    for path_item in path_list.iter() {
        if current_dir.is_empty() {
            current_dir = format!("{}", path_item);
        } else {
            current_dir = format!("{}/{}/", current_dir.clone(), &path_item);
        }
        let option = db_path_cache.get(current_dir.clone().as_str()).await;
        let cache_dir_id = match option {
            Some(option) => option,
            None => "".to_string(),
        };
        if cache_dir_id.is_empty() {
            let mut path_info: PathInfo = PathInfo::default();
            path_info.full_path = current_dir.clone();
            let mut params: HashMap<&str, String> = HashMap::new();
            params.insert("full_path", current_dir.to_string());
            params.insert("bucket_id", bucket_id.to_string());
            let list_path = path_info_rep.dao.query_by_params(params).await?;

            if list_path.len() > 0 {
                path_info = list_path[0].clone();
                finally_id = path_info.id.clone();
            } else {
                params = HashMap::new();
                let current_id = build_snow_id();
                params.insert("id", current_id.to_string());
                if parent_id == "" {
                    root = true;
                }
                params.insert(
                    "root",
                    match root {
                        true => "1".to_string(),
                        false => "0".to_string(),
                    },
                );
                params.insert("bucket_id", bucket_id.to_string());
                params.insert("path", path_item.to_string());
                params.insert("parent", parent_id.to_string());
                params.insert("create_time", build_time().await);
                params.insert("full_path", "".to_owned());
                path_info_rep.dao.insert(params).await?;
                finally_id = current_id;
            }
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
