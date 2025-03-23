use actix_web::{get, web, HttpResponse, Responder};
use common::{AppError, AppState};
use model::{ FileRepository, Repository};
use sqlx::{query_file, MySqlPool};
use std::fs::File;
use std::io::Read;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    cfg.app_data(state.clone()).service(download);
}


/// **大文件流式下载**（`streaming`）
#[utoipa::path(
    get,
    path = "/download/{id}",
    params(("filename" = String, Path, description = "要下载的文件名")),
    responses(
        (status = 200, description = "文件流式下载", content_type = "application/octet-stream"),
    )
)]
#[get("/download/{id}")]
async fn download(
    app_state: web::Data<AppState>,
    id: web::Path<String>,
    file_rep:web::Data<FileRepository>
) -> std::result::Result<impl Responder, AppError> {
    let file_info = file_rep.dao.find_by_id(&*id).await?;
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

