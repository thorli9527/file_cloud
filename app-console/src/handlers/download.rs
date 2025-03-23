use actix_web::{HttpResponse, Responder, get, web};
use common::{AppError, AppState, result_error_msg};
use model::{BucketRepository, FileInfo, FileRepository, Repository, RightType, UserBucketRepository};
use sqlx::{MySqlPool, query_file};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    cfg.app_data(state.clone()).service(download);
}

/// **大文件流式下载**（`streaming`）
#[utoipa::path(
    get,
    path = "/download/{file_id}/{bucket}/{userId}",
    params(("filename" = String, Path, description = "要下载的文件名")),
    responses(
        (status = 200, description = "文件流式下载", content_type = "application/octet-stream"),
    )
)]
#[get("/download/{file_id}/{userId}")]
async fn download(
    params: web::Path<(String, String)>,
    user_bucket_rep: web::Data<UserBucketRepository>,
    bucket_rep: web::Data<BucketRepository>,
    file_rep: web::Data<FileRepository>,
) -> Result<impl Responder, AppError> {
    let (file_id,user_id) = &*params;
    let file_info:FileInfo = match file_rep.dao.find_by_id(file_id).await {
        Ok(file) => file,
        Err(e) => return Err(AppError::NotFound("file.not.found".to_owned())),
    };
    let mut has_right = false;
    let bucket_info = bucket_rep.dao.find_by_id(&file_info.bucket_id).await?;
    if bucket_info.pub_read {
        has_right = true;
    }
    if !has_right {
        let user_bucket_list_right = user_bucket_rep.query_by_user_id(user_id).await?;
        for user_bucket_tmp in &user_bucket_list_right {
            if &user_bucket_tmp.bucket_id == &file_info.bucket_id {
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
    }
    if !has_right {
        return Err(AppError::NotErrorNoRight("no.right".to_owned()));
    }

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
