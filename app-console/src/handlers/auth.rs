use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpMessage as _, HttpRequest, HttpServer, Responder,
    cookie::{Key, time::Duration},
    error,
    http::StatusCode,
    middleware, web,
};
use common::{AppError, BaseResponse, result_data, result_error};
use futures_util::future::err;
use model::UserRepository;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

const ONE_MINUTE: Duration = Duration::minutes(60);
#[derive(Debug, Serialize, Deserialize)]
struct LoginDto {
    username: String,
    password: String,
}

async fn login(
    dto: Json<LoginDto>,
    user_rep: web::Data<UserRepository>,
) -> Result<impl Responder, AppError> {
    match user_rep
        .login(dto.username.to_string(), dto.username.to_string())
        .await
    {
        Ok(info) => Ok(web::Json(result_data(info))),
        _ => {
            return Ok(web::Json(result_error()));
        }
    }
    // Identity::login(&req.extensions(), "user1".to_owned()).unwrap();
}

async fn logout(id: Identity) -> Result<impl Responder, AppError> {
    id.logout();
    Ok(web::Json(BaseResponse::ok_no_result()))
    // web::Redirect::to("/").using_status_code(StatusCode::FOUND)
}
