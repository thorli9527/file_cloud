pub mod common;
mod download;
pub mod upload;

use actix_web::web;
use ::common::AppState;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    common::configure(cfg);
    upload::configure(cfg, state.clone());
}
