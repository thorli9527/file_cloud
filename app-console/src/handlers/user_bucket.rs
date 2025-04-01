use actix_web::web::Data;
use actix_web::{post, web, Responder};
use common::{result, result_list, AppError, AppState, RightType};
use model::{Repository, UserBucketRepository};
use serde::Deserialize;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(user_bucket_list);
}

#[post("/user/bucket/list/{username}")]
async fn user_bucket_list(
    user_name: web::Path<String>,
    user_bucket_reg: web::Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    Ok(web::Json(result_list(user_bucket_reg.query_by_user_name(&*user_name).await?)))
}



#[derive(Debug, Deserialize, Validate,Clone)]
struct UserBucketNew {
    bucket_id: String,
    user_id: String,
    right_type: RightType,
}

#[post("/user/bucket/right/add")]
async fn save(
    data: web::Json<UserBucketNew>,
    user_bucket_rep: Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
     &data.validate();
    user_bucket_rep.change_right(data.user_id.clone(),data.bucket_id.clone(),  data.right_type.clone());
    return Ok(web::Json(result()));
}


#[post("/user/bucket/delete/{id}")]
async fn user_bucket_delete(
    id: web::Path<i64>,
    user_reg: Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    user_reg.dao.del_by_id(*id).await?;
    Ok(web::Json(result()))
}





