pub mod common;
mod download;
pub mod upload;

use actix_web::web;
use ::common::AppState;
use model::biz_repository::UserRepository;
use sqlx::MySqlPool;
use std::sync::Arc;
// use model::UserRepository::UserRepository;

pub  fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>,pool:Arc<MySqlPool>) {
   let user_info= UserRepository::new(pool);
    cfg.app_data(web::Data::new(user_info));
    common::configure(cfg);
    upload::configure(cfg, state.clone());
    // download::configure(cfg, state.clone());
    // cfg.service(
    //     SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone()),
    // );
}
