use actix_web::web::Data;
use actix_web::{Responder, post, web};
use chrono::Local;
use common::*;
use model::*;
use model::{BucketRepository, Repository};
use serde::Deserialize;
use std::collections::HashMap;
use utoipa::ToSchema;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(list);
    cfg.service(delete);
    cfg.service(save);
}
#[utoipa::path(
    post,
    path = "/bucket/list",
    params(
        // ("hash" = String, description = "The hash of the transaction to query")
    ),
    responses(
        (status = 200, description = "successfully",body =Bucket)
    )
)]
#[post("/bucket/list")]
async fn list(
    page: web::Json<PageInfo>,
    bucket_rep: Data<BucketRepository>,
) -> std::result::Result<impl Responder, AppError> {
    let params: HashMap<&str, String> = HashMap::new();
    let page_result = bucket_rep.dao.query_by_page(params, &page).await?;
    Ok(web::Json(result_page(page_result)))
}

#[utoipa::path(
    post,
    path = "/bucket/delete/{id}",
    params(
        // ("hash" = String, description = "The hash of the transaction to query")
    ),
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/bucket/delete/{id}")]
async fn delete(
    id: web::Path<String>,
    bucket_rep: Data<BucketRepository>,
) -> std::result::Result<impl Responder, AppError> {
    let id_p = format!("{}", id);
    bucket_rep.dao.del_by_id(id_p).await?;
    Ok(web::Json(result()))
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
struct BucketSaveDto {
    id: String,
    #[validate(length(min = 3, max = 32))]
    name: String,
    quota: i32,
    pub_read: bool,
    pub_write: bool,
}
#[utoipa::path(
    post,
    path = "/bucket/save",
    responses(
        (status = 200, description = "successfully",body = Bucket)
    )
)]
#[post("/bucket/save")]
async fn save(
    bucket_rep: Data<BucketRepository>,
    data: web::Json<BucketSaveDto>,
) -> std::result::Result<impl Responder, AppError> {
    if let Err(e) = &data.validate() {
        let msg = format!("Validation failed: {:?}", e);
        return Ok(web::Json(result_error_msg(msg.as_str())));
    }
    let mut params: HashMap<&str, String> = HashMap::new();
    if (data.id.is_empty()) {
        params.insert("id", build_id());
        params.insert("current_quota", "0".to_owned());
        let now = Local::now();
        params.insert("create_time", now.format("%Y-%m-%d %H:%M:%S").to_string());
    }
    params.insert("name", data.name.clone());
    params.insert("quota", data.quota.to_string());
    params.insert(
        "pub_read",
        match &data.pub_read {
            true => "1".to_string(),
            false => "0".to_string(),
        },
    );
    params.insert(
        "pub_write",
        match &data.pub_write {
            true => "1".to_string(),
            false => "0".to_string(),
        },
    );

    if (data.id.is_empty()) {
        bucket_rep.dao.insert(params).await?;
    } else {
        bucket_rep.dao.change(&data.id, params).await?;
    }

    return Ok(web::Json(result()));
}
