use actix_web::web::Data;
use actix_web::{cookie::time::Duration, post, web, HttpRequest, Responder};
use common::{
    build_id, result_data, AppError, AppState, BaseResponse, BucketCache, UserCache,
};
use model::UserRepository;
use model::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

const ONE_MINUTE: Duration = Duration::minutes(60);

pub fn configure(cfg: &mut web::ServiceConfig, state: Data<AppState>) {
    cfg.app_data(state.clone());
    cfg.service(login);
    cfg.service(logout);
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub user_name: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult<'a> {
    pub user_name: &'a str,
    pub token: &'a str,
}
#[post("/auth/login")]
pub async fn login(
    dto: web::Json<LoginInfo>,
    state: web::Data<AppState>,
    user_rep: web::Data<UserRepository>,
    user_bucket_rep: web::Data<UserBucketRepository>,
) -> Result<impl Responder, AppError> {
    let result = user_rep.login(&dto.user_name, &dto.password).await?;
    let session_id = &build_id();
    let bucket_list = user_bucket_rep.query_by_user_id_and_bucket_Id(&result.id,&result.id).await?;
    let mut bucket_cache_list: Vec<BucketCache> = Vec::new();
    for bucket in bucket_list {
        bucket_cache_list.push(BucketCache {
            right_id:bucket.id,
            bucket_id: bucket.bucket_id,
            name: bucket.user_name.clone(),
            right_type: bucket.user_right,
        });
    }
    let user_cache = UserCache {
        id: result.id,
        is_admin: result.is_admin.clone(),
        user_name: result.user_name.clone(),
        bucket_list:vec![],
    };
    state.session_cache.insert(session_id.clone(),user_cache).await;
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
            if auth_str.starts_with("Bearer ") {
                let token_key = &auth_str[9..];
                state.session_cache.remove(token_key);
            }
        }
    }
    Ok(web::Json(BaseResponse::ok_no_result()))
}
