use actix_web::web::Data;
use actix_web::{post, web, Responder};
use common::{result, result_list, AppError, AppState};
use model::{Repository, UserBucketRepository};
use serde::Deserialize;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());

}







#[post("/user/bucket/delete/{id}")]
async fn user_bucket_delete(
    id: web::Path<i64>,
    user_reg: Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    user_reg.dao.del_by_id(*id).await?;
    Ok(web::Json(result()))
}





