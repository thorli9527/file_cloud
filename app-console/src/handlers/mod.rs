mod auth;
use auth::*;
mod bucket;
mod common;
mod download;
mod upload;
mod user;
mod user_bucket;
use actix_web::web;
use ::common::AppState;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    common::configure(cfg);
    user::configure(cfg, state.clone());
    bucket::configure(cfg, state.clone());
    user_bucket::configure(cfg, state.clone());
    auth::configure(cfg, state.clone());
    upload::configure(cfg, state.clone());
    download::configure(cfg, state.clone());
}

