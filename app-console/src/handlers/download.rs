use std::collections::HashMap;
use actix_web::{get, web, HttpResponse, Responder};
use common::{result_error_msg, AppError, AppState};
use model::{BucketRepository, FileRepository, Repository, RightType, UserBucketRepository};
use sqlx::{query_file, MySqlPool};
use std::fs::File;
use std::io::Read;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    cfg.app_data(state.clone()).service(download);
}


/// **大文件流式下载**（`streaming`）
#[utoipa::path(
    get,
    path = "/download/{bucket}/{userId}/{file_id}",
    params(("filename" = String, Path, description = "要下载的文件名")),
    responses(
        (status = 200, description = "文件流式下载", content_type = "application/octet-stream"),
    )
)]
#[get("/download/{bucket}/{userId}/{file_id}")]
async fn download(
    params: web::Path<(String, String,String)>,
    app_state: web::Data<AppState>,
    user_bucket_rep: web::Data<UserBucketRepository>,
    bucket_rep: web::Data<BucketRepository>,
    file_rep:web::Data<FileRepository>
) -> std::result::Result<impl Responder, AppError> {
    let (bucket, user_id,file_id) = &*params;
    let user_bucket_list_right = user_bucket_rep.query_by_user_id(user_id).await?;
    let mut has_right = false;
    for user_bucket_tmp in &user_bucket_list_right {
        if user_bucket_tmp.name == *bucket {
            match &user_bucket_tmp.right {
                RightType::Write => {
                    has_right = false;
                }
                (item) => {
                    has_right = true;
                }
            }
        }
    }
    if !has_right {
        let mut parmas: HashMap<&str, String> = HashMap::new();
        parmas.insert("name", bucket.clone());
        let bucket_list = bucket_rep.dao.query_by_params(parmas).await?;
        if bucket_list.len() > 0 {
            let bucket_info = bucket_list[0].clone();
            if bucket_info.pub_read {
                has_right = true;
            }
        }
    }

    if !has_right {
        return Err(AppError::NotErrorNoRight("no.right".to_owned()));
    }

    let file_info = file_rep.dao.find_by_id(file_id).await?;
    let item_files = file_info.items;
    let mut buffer = Vec::new();
    for item in item_files.iter() {
        let mut item_file = File::open(&item.path).unwrap();
        item_file.read_to_end(&mut buffer).unwrap();
    }
    Ok(HttpResponse::Ok()
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file_info.file_name),
        ))
        .body(buffer))
}

