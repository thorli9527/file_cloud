use actix_web::web::Data;
use actix_web::{post, web, App, HttpRequest, Responder};
use chrono::{Local, NaiveDateTime};
use common::{build_snow_id, build_time, get_session_user, result, result_data, result_list, AppError, AppState, OrderType};
use model::date_format::date_format;
use model::{BucketRepository, FileInfo, FileRepository, FileType, ImageType, PathDelTask, PathDelTaskRepository, PathRepository, QueryParam, Repository, UserBucketRepository, UserRepository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::error;
use tokio::fs;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.service(file_list);
    cfg.service(file_path_info);
    cfg.service(mkdir);
    cfg.service(file_del);
    cfg.service(del_path);
}

#[post("/file/mkdir")]
pub async fn mkdir(dto: web::Json<PathNewDao>,
                   app_state: web::Data<AppState>,
                   path_info_rep: web::Data<PathRepository>,
                   user_bucket_rep: web::Data<UserBucketRepository>,
                   req: HttpRequest, ) -> Result<impl Responder, AppError> {
    if (dto.parent == 0 && dto.path.is_empty()) {
        return Err(AppError::InvalidInput("invalid.params".to_string()));
    }
    let user = get_session_user(&app_state, req).await?;
    let mut right = false;
    if user.is_admin {
        right = true;
    }
    if !right {
        let user_bucket = user_bucket_rep.query_by_user_id_and_bucket_Id(&user.id, &dto.bucket_id).await?;
        if user_bucket.is_empty() {
            return Err(AppError::NoRight("no.right".to_string()));
        }
        match &user_bucket[0].user_right {
            1 => {
                right = true;
            }
            2 => {
                right = true;
            }
            _ => {
                return Err(AppError::NoRight("no.right".to_string()))
            }
        }
    }
    if !right {
        return Err(AppError::NoRight("no.right".to_string()));
    }
    path_info_rep.new_path(&dto.path, &dto.parent, &dto.bucket_id).await?;
    Ok(web::Json(result()))
}


#[post("/file/path/{id}")]
async fn file_path_info(path_id: web::Path<i64>, path_rep: Data<PathRepository>) -> Result<impl Responder, AppError> {
    let result = path_rep.dao.find_by_id(*path_id).await?;
    Ok(web::Json(result_data(result)))
}
#[post("/file/del_path/{path_id}")]
async fn del_path(
    path_id: web::Path<i64>,
    req: HttpRequest,
    state: web::Data<AppState>,
    path_rep: Data<PathRepository>,
    path_del_task_rep: Data<PathDelTaskRepository>,
    user_bucket_rep: Data<UserBucketRepository>, ) -> Result<impl Responder, AppError> {
    let user = get_session_user(&state, req).await?;
    let mut right = false;
    if user.is_admin {
        right = true;
    }
    let path_info = path_rep.dao.find_by_id(*path_id).await?;
    if !right {
        let user_bucket = user_bucket_rep.query_by_user_id_and_bucket_Id(&user.id, &path_info.bucket_id).await?;
        if user_bucket.is_empty() {
            return Err(AppError::NoRight("no.right".to_string()));
        }
        match &user_bucket[0].user_right {
            1 => {
                right = true;
            }
            2 => {
                right = true;
            }
            _ => {
                return Err(AppError::NoRight("no.right".to_string()))
            }
        }
    }
    if !right {
        return Err(AppError::NoRight("no.right".to_string()));
    }
    let now = Local::now();
    let path_del_task = PathDelTask { id: build_snow_id(), path_id: *path_id, del_file_status: false, del_path_status: false, create_time: now.naive_local() };
    path_del_task_rep.create(path_del_task, &path_rep).await?;
    Ok(web::Json(result()))
}
#[post("/file/delete/{file_id}")]
async fn file_del(
    file_id: web::Path<i64>,
    req: HttpRequest,
    state: web::Data<AppState>,
    file_rep: Data<FileRepository>,
    bucket_rep: web::Data<BucketRepository>,
    user_bucket_rep: Data<UserBucketRepository>, ) -> Result<impl Responder, AppError>
{
    let file_info: FileInfo = match file_rep.dao.find_by_id(*file_id).await {
        Ok(file) => file,
        Err(e) => return Err(AppError::NotFound("file.not.found".to_owned())),
    };
    let mut has_right = false;
    let bucket_info = bucket_rep.dao.find_by_id(file_info.bucket_id).await?;
    if bucket_info.pub_read {
        has_right = true;
    }

    if !has_right {
        let user_id = get_session_user(&state, req).await?.id;
        let user_bucket_list_right = user_bucket_rep.query_by_user_id_and_bucket_Id(&user_id, &bucket_info.id).await?;
        for user_bucket_tmp in &user_bucket_list_right {
            if &user_bucket_tmp.bucket_id == &file_info.bucket_id {
                match &user_bucket_tmp.user_right {
                    1 => {
                        has_right = false;
                    }
                    2 => {
                        has_right = false;
                    }
                    (item) => {
                        has_right = true;
                    }
                }
            }
        }
    }
    if !has_right {
        return Err(AppError::NoRight("no.right".to_owned()));
    }
    let item_files = file_info.items;
    for file_item in item_files.iter() {
        let msg = match fs::remove_file(&file_item.path).await {
            Ok(_) => "",
            Err(e) => &e.to_string(),
        };
        if !msg.is_empty() {
            error!("delete file {} error: {}", file_item.path, msg);
        }
    }
    file_rep.dao.del_by_id(*file_id).await?;
    Ok(web::Json(result()))
}

#[post("/file/list")]
async fn file_list(
    query: web::Json<PathQuery>,
    file_rep: Data<FileRepository>,
    path_rep: Data<PathRepository>,
    user_bucket_rep: Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    let mut params=vec![];
    // let mut params: HashMap<&str, String> = HashMap::new();
    if (query.path_id == 0) {
        // params.insert("root", "1".to_owned());
        params.push(QueryParam::eq("root", "1".to_owned().as_str()));
    }
    let mut result_list: Vec<FileResult> = Vec::new();
    params.push(QueryParam::eq("bucket_id", query.bucket_id.to_string().as_str()));
    if query.query_type == QueryDataType::DIR {
        let mut path_params = params.clone();
        if (query.path_id != 0) {
            path_params.push(QueryParam::eq("parent", query.path_id.to_string().as_str()));
        }
        let mut path_query=params.clone();
        if query.search_key.is_some() {
            match &query.search_key{
                Some(key) => {
                    path_query.push(QueryParam::like_end("path", key));
                }
                None => {}
            }
        }
        path_query.push(QueryParam::eq("bucket_id", query.bucket_id.to_string().as_str()));
        path_query.push(QueryParam::eq("parent", query.path_id.to_string().as_str()));
        let path_list = path_rep.dao.query_by_max_id(query.max_id,path_query, OrderType::ASC, &query.page_size).await?;
        for item in path_list {
            let path_file_name = format!("{}{}", &item.full_path, "/");
            let x = file_rep.path_size(&path_file_name).await?;
            let file = FileResult {
                id: item.id,
                bucket_id: item.bucket_id,
                file_name: item.path,
                file_type: FileType::DIR,
                size: x.clone() as u32,
                image_type: ImageType::NONE,
                create_time: item.create_time,
            };
            result_list.push(file);
        }
    }
    let current_data_size = result_list.len() as i64;
    let page_size = query.page_size as i64;
    let mut file_max_id = 0;
    if query.query_type == QueryDataType::FILE {
        file_max_id = query.max_id;
    }
    if current_data_size < page_size {
        params.push(QueryParam::eq("path_ref", query.path_id.to_string().as_str()));
        if query.search_key.is_some() {
            match &query.search_key{
                Some(key) => {
                    params.push(QueryParam::like_end("name", key));
                }
                None => {}
            }
        }
        let limit_size = (page_size - current_data_size) as i16;
        let file_list = file_rep
            .dao
            .query_by_max_id(file_max_id, params, OrderType::ASC, &limit_size)
            .await?;
        for item in file_list {
            let file = FileResult {
                id: item.id,
                bucket_id: item.bucket_id,
                file_name: item.name,
                file_type: item.file_type,
                size: item.size,
                image_type: ImageType::NONE,
                create_time: item.create_time,
            };
            result_list.push(file);
        }
    }
    Ok(web::Json(result_data(result_list)))
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum QueryDataType {
    FILE,
    DIR,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathQuery {
    path_id: i64,
    bucket_id: i64,
    page_size: i16,
    query_type: QueryDataType,
    search_key: Option<String>,
    max_id: i64,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    id: i64,
    file_name: String,
    file_type: FileType,
    bucket_id: i64,
    size: u32,
    image_type: ImageType,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathNewDao {
    bucket_id: i64,
    parent: i64,
    path: String,
}