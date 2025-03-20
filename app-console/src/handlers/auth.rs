use actix_web::web::Data;
use actix_web::{cookie::time::Duration, post, web, Responder};
use common::{build_id, result_data, result_error, AppError, AppState, BaseResponse};
use model::UserRepository;
use model::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;
use utoipa::ToSchema;

const ONE_MINUTE: Duration = Duration::minutes(60);


pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(login);
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}
#[utoipa::path(
    post,
    path = "/auth/login",
    responses(
        (status = 200, description = "successfully",body = String)
    )
)]
#[post("/auth/login")]
pub async fn login(
    dto: web::Json<LoginInfo>,
    state: web::Data<AppState>,
    user_rep: web::Data<UserRepository>,
) -> Result<impl Responder, AppError> {
    let result = user_rep
        .login(dto.username.to_string(), dto.password.to_string())
        .await;
    match result
    {
        Ok(info) => {
            let session_id = build_id();
            state.session_cache.insert(session_id, info.id.clone());
            Ok(web::Json(result_data(info.clone())))
        }
        Err(e) => {
           return Ok(web::Json(result_error(e)));
        }
    }
}

async fn logout(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    Ok(web::Json(BaseResponse::ok_no_result()))
    // web::Redirect::to("/").using_status_code(StatusCode::FOUND)
}
