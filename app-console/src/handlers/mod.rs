pub mod auth;
pub use auth::*;
pub mod bucket;
pub mod common;
mod download;
pub mod upload;
pub mod user;
pub mod user_bucket;
use actix_web::web;
use ::common::AppState;
use utoipa::OpenApi;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    common::configure(cfg);
    user::configure(cfg, state.clone());
    bucket::configure(cfg, state.clone());
    user_bucket::configure(cfg, state.clone());
    auth::configure(cfg, state.clone());
    upload::configure(cfg, state.clone());
    download::configure(cfg, state.clone());

}

