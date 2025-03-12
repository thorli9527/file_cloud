use actix_web::{get, post, web, HttpResponse, Responder};
use crate::{AppError, BaseResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(status);
}

#[utoipa::path(
    post,
    path = "/status",
    responses(
        (status = 200, description = "successfully")
    )
)]
#[post("/status")]
pub async fn status() ->Result<impl Responder,AppError> {
    Ok(web::Json(BaseResponse::ok_no_result()))
}