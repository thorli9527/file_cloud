use actix_web::web::Data;
use actix_web::{post, web, Responder};
use common::{result, result_error_msg, result_list, AppError, AppState};
use model::{BucketInfoResult, Repository, RightType, UserBucketRepository};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(user_bucket_list);
}

#[utoipa::path(
    post,
    path = "/user/bucket/list/{username}",
    params(
    ),
    responses(
        (status = 200, description = "successfully",body = BucketInfoResult)
    )
)]
#[post("/user/bucket/list/{username}")]
async fn user_bucket_list(
    user_name: web::Path<String>,
    user_bucket_reg: web::Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    Ok(web::Json(result_list(user_bucket_reg.query_by_user_name(&*user_name).await?)))
}



#[derive(Debug, Deserialize, ToSchema, Validate,Clone)]
struct UserBucketNew {
    bucket_id: String,
    user_id: String,
    right_type: RightType,
}
#[utoipa::path(
    post,
    path = "/user/bucket/add",
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/user/bucket/right/add")]
async fn save(
    data: web::Json<UserBucketNew>,
    user_bucket_rep: Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    if let Err(e) = &data.validate() {
        let msg = format!("Validation failed: {:?}", e);
        return Ok(web::Json(result_error_msg(msg.as_str())));
    }
    user_bucket_rep.change_right(data.user_id.clone(),data.bucket_id.clone(),  data.right_type.clone());
    return Ok(web::Json(result()));
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





