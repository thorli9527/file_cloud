use actix_web::web::Data;
use actix_web::{post, web, HttpRequest, Responder};
use chrono::NaiveDateTime;
use common::{get_session_user, result, result_data, AppError, AppState, OrderType, RightType};
use model::date_format::date_format;
use model::{
    FileRepository, FileType, ImageType, PathRepository, Repository, UserBucketRepository,
    UserRepository,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.service(file_list);
    cfg.service(file_path_info);
    cfg.service(mkdir);
}

#[post("/file/mkdir")]
pub async fn mkdir(dto: web::Json<PathNewDao>,
                   app_state: web::Data<AppState>,
                   path_info_rep: web::Data<PathRepository>,
                   user_bucket_rep: web::Data<UserBucketRepository>,
                   req: HttpRequest, ) -> Result<impl Responder, AppError> {
    if(dto.parent==0 && dto.path.is_empty()) {
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
            return Err(AppError::NoRight("no.right".to_string()))
        }
        match &user_bucket[0].right {
            RightType::Write => {
                right = true;
            }
            RightType::ReadWrite => {
                right = true;
            }
            _ => {
                return Err(AppError::NoRight("no.right".to_string()))
            }
        }
    }
    if !right {
        return Err(AppError::NoRight("no.right".to_string()))
    }
    path_info_rep.new_path(&dto.path,&dto.parent,&dto.bucket_id).await?;
    Ok(web::Json(result()))
}


#[post("/file/path/{id}")]
async  fn file_path_info(path_id:web::Path<i64>, path_rep: Data<PathRepository>,)-> Result<impl Responder, AppError> {
    let result = path_rep.dao.find_by_id(*path_id).await?;
    Ok(web::Json(result_data(result)))
}

#[post("/file/list")]
async fn file_list(
    query: web::Json<PathQuery>,
    user_reg: Data<UserRepository>,
    file_rep: Data<FileRepository>,
    path_rep: Data<PathRepository>,
    user_bucket_rep: Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    let mut params: HashMap<&str, String> = HashMap::new();
    if (query.path_id == 0) {
        params.insert("root", "1".to_owned());
    }
    let mut result_list: Vec<FileResult> = Vec::new();
    params.insert("bucket_id", query.bucket_id.to_string());
    if query.query_type == QueryDataType::DIR {
        let mut path_params = params.clone();
        if (query.path_id != 0) {
            path_params.insert("parent", query.path_id.to_string());
        }
        let path_list = path_rep
            .dao
            .query_by_max_id(query.max_id, path_params, OrderType::ASC, &query.page_size)
            .await?;
        for item in path_list {
            let path_file_name=format!("{}{}",&item.full_path,"/");
            let x = file_rep.path_size(&path_file_name).await?;
            let file = FileResult {
                id: item.id,
                bucket_id:item.bucket_id,
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
        params.insert("path_ref", query.path_id.to_string());
        let limit_size = (page_size - current_data_size) as i16;
        let file_list = file_rep
            .dao
            .query_by_max_id(file_max_id, params.clone(), OrderType::ASC, &limit_size)
            .await?;
        for item in file_list {
            let file = FileResult {
                id: item.id,
                bucket_id:item.bucket_id,
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
    max_id: i64,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    id: i64,
    file_name: String,
    file_type: FileType,
    bucket_id:i64,
    size: u32,
    image_type: ImageType,
    #[serde(with = "date_format")]
    pub create_time: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathNewDao{
    bucket_id:i64,
    parent:i64,
    path:String,
}