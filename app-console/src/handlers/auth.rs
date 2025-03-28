use actix_web::web::Data;
use actix_web::{HttpRequest, Responder, cookie::time::Duration, post, web};
use common::{AppError, AppState, BaseResponse, build_id, result_data,};
use log::info;
use model::UserRepository;
use model::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

const ONE_MINUTE: Duration = Duration::minutes(60);

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(login);
    cfg.service(logout);
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub user_name: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult<'a> {
    pub user_name: &'a str,
    pub token: &'a str,
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
    let result = user_rep.login(&dto.user_name, &dto.password).await?;
    let session_id = &build_id();
    state.session_cache.insert(session_id.clone(), result.id.clone()).await;
    Ok(web::Json(result_data(LoginResult {
        user_name: &result.user_name,
        token: &session_id,
    })))
}
#[post("/auth/logout")]
async fn logout(state: web::Data<AppState>, req: HttpRequest) -> Result<impl Responder, AppError> {
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
