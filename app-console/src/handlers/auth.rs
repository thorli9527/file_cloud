use actix_web::web::Data;
use actix_web::{cookie::time::Duration, post, web, HttpRequest, Responder};
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
    cfg.service(logout);
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
#[post("/auth/logout")]
async fn logout(state: web::Data<AppState>,req: HttpRequest) -> Result<impl Responder, AppError> {
    let auth_header = req.headers().get("Authorization");
    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Token ") {
                let token_key = &auth_str[8..];
                state.session_cache.remove(token_key);
            }
        }
    }
    Ok(web::Json(BaseResponse::ok_no_result()))
}
