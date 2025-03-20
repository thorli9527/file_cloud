// use actix_web::{get, web, HttpResponse, Responder};
// use common::{AppError, AppState};
// use model::FileInfoTem;
// use sqlx::MySqlPool;
// use std::fs::File;
// use std::io::Read;
//
// pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
//     cfg.app_data(state.clone()).service(download);
// }
//
// #[get("/users/{id}")]
// async fn get_user(id: web::Path<u32>) -> impl Responder {
//     HttpResponse::Ok().body(format!("User ID: {}", id))
// }
// /// **大文件流式下载**（`streaming`）
// #[utoipa::path(
//     get,
//     path = "/download/{filename}",
//     params(("filename" = String, Path, description = "要下载的文件名")),
//     responses(
//         (status = 200, description = "文件流式下载", content_type = "application/octet-stream"),
//     )
// )]
// #[get("/download/{id}")]
// async fn download(
//     app_state: web::Data<AppState>,
//     id: web::Path<String>,
// ) -> std::result::Result<impl Responder, AppError> {
//     let file_info = query_file(&app_state.pool, id.clone()).await?;
//     let item_files = file_info.items.split(",");
//     let mut buffer = Vec::new();
//     for item in item_files {
//         if item.is_empty() {
//             continue;
//         }
//         let mut item_file = File::open(&item).unwrap();
//         item_file.read_to_end(&mut buffer).unwrap();
//     }
//     Ok(HttpResponse::Ok()
//         .append_header((
//             "Content-Disposition",
//             format!("attachment; filename=\"{}\"", file_info.file_name),
//         ))
//         .body(buffer))
// }
// async fn query_file(conn: &MySqlPool, id: String) -> Result<FileInfoTem, AppError> {
//     let list_db=  sqlx::query_as::<_, FileInfoTem>(
//         "SELECT id,file_name, file_type, image_type, items, thumbnail, size, thumbnail_status FROM file_info where id=?"
//     )
//         .bind(id)  // ✅ 绑定参数
//         .fetch_all(conn)
//         .await.unwrap();
//     if list_db.is_empty() {
//         return Err(AppError::InternalError("file.not.found".to_string()));
//     }
//     return Ok(list_db[0].clone());
// }
