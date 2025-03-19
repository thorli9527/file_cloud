use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use actix_web::cookie::time::Duration;
use actix_web::http::StatusCode;
use actix_web::middleware::Identity;
use actix_web::web::{service, Data};
use serde::{Deserialize, Serialize};
use common::{ result, result_list, AppError, AppState, BaseResponse};
use model::{Repository, UserInfo, UserRepository};

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(user_list);
    cfg.service(user_delete);
    cfg.service(user_save);
    cfg.service(user_change);
    cfg.service(user_change_password);
}
#[utoipa::path(
    post,
    path = "/user/list",
    params(
       // ("hash" = String, description = "The hash of the transaction to query")
    ),
    responses(
        (status = 200, description = "successfully",body = UserInfo)
    )
)]
#[post("/user/list")]
pub async fn user_list(user_reg:web::Data<UserRepository>) -> Result<impl Responder, AppError> {
    let user_list_result=user_reg.dao.get_all().await?;
    Ok(web::Json(result_list(user_list_result)))
}

#[post("/user/delete")]
async fn user_delete(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    Ok(web::Json(result()))
}
#[post("/user/save")]
async fn user_save(data: web::Data<AppState>,user: web::Json<UserInfo>) -> Result<impl Responder, AppError> {
    Ok(web::Json(result()))
}
#[post("/user/change")]
async fn user_change(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    Ok(web::Json(result()))
}

#[post("/user/change/password")]
async fn user_change_password(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    Ok(web::Json(result()))
}