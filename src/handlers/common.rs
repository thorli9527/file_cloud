use crate::{AppError, BaseResponse};
use actix_web::{post, web, Responder};

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