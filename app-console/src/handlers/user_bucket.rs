use actix_web::web::{Data, Path};
use actix_web::{post, web, Responder};
use serde::Deserialize;
use common::{
    result, result_list, AppError,
    AppState,
};
use model::{Repository, RightType, UserBucketRepository, UserInfo, UserRepository,BucketInfoResult};
use utoipa::ToSchema;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(list);
}

#[utoipa::path(
    post,
    path = "/user/bucket/list",
    params(
    ),
    responses(
        (status = 200, description = "successfully",body = BucketInfoResult)
    )
)]
#[post("/user/bucket/list/{username}")]
async fn list(user_name: web::Path<String>,user_bucket_reg: web::Data<UserBucketRepository>) -> Result<impl Responder, AppError> {
    let user_bucket_list = user_bucket_reg.find_by_user_name(&*user_name).await?;
    Ok(web::Json(result_list(user_bucket_list)))
}

#[utoipa::path(
    post,
    path = "/user/bucket/delete/{id}",
    params(
        // ("hash" = String, description = "The hash of the transaction to query")
    ),
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/user/bucket/delete/{id}")]
async fn user_bucket_delete(
    id: web::Path<String>,
    user_reg: Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    let id_p = format!("{}", id);
    user_reg.dao.del_by_id(id_p).await?;
    Ok(web::Json(result()))
}