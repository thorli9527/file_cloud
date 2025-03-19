use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpMessage as _, HttpRequest, HttpServer, Responder,
    cookie::{Key, time::Duration},
    error,
    http::StatusCode,
    middleware, web,
};
use futures_util::future::err;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use common::{AppError, BaseResponse};

const ONE_MINUTE: Duration = Duration::minutes(60);
#[derive(Debug, Serialize, Deserialize)]
struct LoginDto {
    username: String,
    password: String,
}
async fn index(identity: Option<Identity>) -> Result<impl Responder,AppError> {
    // let id = match identity.map(|id| id.id()) {
    //     Some(Ok(id)) => id,
    //     ()=> return Ok(web::Json(BaseResponse::err_result_msg("token.with.out"))),
    // };
    Ok(web::Json(BaseResponse::ok_result_msg("token")))
}

async fn login(dto: Json<LoginDto>) -> Result<impl Responder, AppError> {
    // Identity::login(&req.extensions(), "user1".to_owned()).unwrap();
    Ok(web::Json(BaseResponse::ok_result_msg("token")))
}

async fn logout(id: Identity) -> Result<impl Responder ,AppError>{
    id.logout();
    Ok(web::Json(BaseResponse::ok_no_result()))
    // web::Redirect::to("/").using_status_code(StatusCode::FOUND)
}
