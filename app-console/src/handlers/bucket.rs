use actix_web::web::Data;
use actix_web::{post, web, Responder};
use chrono::Local;
use common::*;
use model::*;
use model::{BucketRepository, Repository};
use serde::Deserialize;
use std::collections::HashMap;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(list);
    cfg.service(bucket_delete);
    cfg.service(save);
}
#[post("/bucket/list")]
async fn list(
    page: web::Json<PageInfo>,
    bucket_rep: Data<BucketRepository>,
) -> std::result::Result<impl Responder, AppError> {
    let params: HashMap<&str, String> = HashMap::new();
    let page_result = bucket_rep.dao.query_by_page(params, &page).await?;
    Ok(web::Json(result_page(page_result)))
}


#[post("/bucket/delete/{id}")]
async fn bucket_delete(
    id: web::Path<String>,
    bucket_rep: Data<BucketRepository>,
) -> std::result::Result<impl Responder, AppError> {
    let n_id=id.parse().unwrap();
    bucket_rep.dao.del_by_id(n_id).await?;
    Ok(web::Json(result()))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct BucketSaveDto {
    id: i64,
    #[validate(length(min = 1, max = 32))]
    name: String,
    quota: i32,
    pub_read: bool,
    pub_write: bool,
}
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
    if (data.id==0) {
        params.insert("id", build_snow_id().to_string());
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

    if (data.id==0) {
        bucket_rep.dao.insert(params).await?;
    } else {
        bucket_rep.dao.change(data.id, params).await?;
    }

    return Ok(web::Json(result()));
}
