use actix_web::{post, web, Responder};
use common::{AppError, BaseResponse};

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
pub async fn status() -> Result<impl Responder, AppError> {
    let s = String::from("OK");
    Ok(web::Json(BaseResponse::ok_no_result()))
}
