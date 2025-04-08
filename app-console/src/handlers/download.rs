use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use common::{do_zip, get_session_user, AppError, AppState};
use model::{
    BucketRepository, FileInfo, FileRepository, PathInfo, PathRepository, Repository,
    UserBucketRepository,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use actix_web::http::header;
use async_stream::stream;
use hex::encode;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::io::ReaderStream;
pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    cfg.app_data(state.clone()).service(download);
    cfg.app_data(state.clone()).service(download_path);
}

#[get("/downloadPath/{bucket_id}/{path_id}")]
async fn download_path(
    req: HttpRequest,
    state: web::Data<AppState>,
    params: web::Path<(i64, i64)>,
    user_bucket_rep: web::Data<UserBucketRepository>,
    path_rep: web::Data<PathRepository>,
    file_rep: web::Data<FileRepository>,
) -> Result<impl Responder, AppError> {
    let user = get_session_user(&state, req).await?;
    let (bucket_id, path_id) = params.into_inner();
    let mut has_right = false;
    if (user.is_admin) {
        has_right = true;
    }
    if (!has_right) {
        let user_bucket_list = user_bucket_rep.query_by_user_id_and_bucket_Id(&user.id,&bucket_id).await?;
        for user_bucket in user_bucket_list {
            if user_bucket.bucket_id == bucket_id {
                match &user_bucket.user_right {
                    0 => {
                        has_right = true;
                        break;
                    }
                    2 => {
                        has_right = true;
                        break;
                    }
                    _ => {
                        has_right = false;
                    }
                }
            }
        }
    }

    if !has_right {
        return Err(AppError::NoRight("no.right".to_string()));
    }

    let path_info: PathInfo = path_rep.dao.find_by_id(path_id).await?;

    // 创建临时目录
    let temp_dir = tempdir().map_err(|e| AppError::InternalError(e.to_string()))?;
    let mut max_id = 0 as i64;
    let mut file_list: Vec<FileInfo> = Vec::new();
    let mut create_dir_map: HashMap<&String, bool> = HashMap::new();
    let mut current_list = file_rep.path_file_list(&path_info.full_path, max_id, bucket_id).await?;
    while current_list.len() > 0 {
        file_list.extend(current_list.clone());
        let info = current_list.last().unwrap();
        max_id = info.id;
        current_list = file_rep.path_file_list(&path_info.full_path, max_id, bucket_id).await?
    }
    for file in file_list.iter() {
        let data_dir = &temp_dir.path().join(&file.full_path);
        let option = create_dir_map.get(&file.full_path);
        if option.is_none() {
            fs::create_dir_all(&data_dir)?;
            create_dir_map.insert(&file.full_path, true);
        }

        let output_path = data_dir.join(&file.name);
        let mut output_file = File::create(&output_path).await?;
        for item in file.items.iter() {
            let path = Path::new(&item.path);
            if !path.exists(){
                return Err(AppError::InternalError("file not exists".to_owned()));
            }
            let mut chunk_file = File::open(&*item.path).await?;
            let mut buffer = Vec::new();
            chunk_file.read_to_end(&mut buffer).await?;
            output_file.write_all(&buffer).await?;
        }
    }
    let mut out_file = tempdir().map_err(|e| AppError::InternalError(e.to_string()))?;
    let download_file = out_file.path().join("default.zip");
    do_zip(temp_dir.path().as_ref(), download_file.as_path(), zip::CompressionMethod::Deflated).map_err(|e| AppError::InternalError(e.to_string()));

    let file = File::open(download_file).await?;
    let stream = ReaderStream::new(file);
    Ok(HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "application/zip"))
        .append_header((header::CONTENT_DISPOSITION, "attachment; filename=\"default.zip\""))
        .streaming(stream))
}

/// **大文件流式下载**（`streaming`）
#[get("/download/{file_id}")]
async fn download(
    req: HttpRequest,
    state: web::Data<AppState>,
    file_id: web::Path<i64>,
    user_bucket_rep: web::Data<UserBucketRepository>,
    bucket_rep: web::Data<BucketRepository>,
    file_rep: web::Data<FileRepository>,
) -> Result<impl Responder, AppError> {
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
        let user_bucket_list_right = user_bucket_rep.query_by_user_id_and_bucket_Id(&user_id,&bucket_info.id).await?;
        for user_bucket_tmp in &user_bucket_list_right {
            if &user_bucket_tmp.bucket_id == &file_info.bucket_id {
                match &user_bucket_tmp.user_right {
                    1 => {
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
    let mut buffer = Vec::new();
    for item in item_files.iter() {
        let mut item_file = File::open(&item.path).await.unwrap();
        item_file.read_to_end(&mut buffer).await?;
    }
    let accachement_name=format!("attachment; filename={}",&file_info.name);
    Ok(HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "application/zip"))
        .append_header((header::CONTENT_DISPOSITION, accachement_name))
        .body(buffer))
}
