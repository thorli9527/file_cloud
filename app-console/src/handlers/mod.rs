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
use model::biz_repository::UserRepository;
use model::{BucketRepository, PathRepository, UserBucketRepository};
use sqlx::MySqlPool;
use std::sync::Arc;
use utoipa::OpenApi;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>, pool: Arc<MySqlPool>) {
    let user_info: UserRepository = UserRepository::new(pool.clone());
    let path_info: PathRepository = PathRepository::new(pool.clone());
    let bucket_rep: BucketRepository = BucketRepository::new(pool.clone());
    let user_bucket_rep: UserBucketRepository = UserBucketRepository::new(pool.clone());
    let user_bucket_right: UserBucketRepository = UserBucketRepository::new(pool.clone());
    cfg.app_data(web::Data::new(user_info));
    cfg.app_data(web::Data::new(path_info));
    cfg.app_data(web::Data::new(bucket_rep));
    cfg.app_data(web::Data::new(user_bucket_rep));
    cfg.app_data(web::Data::new(user_bucket_right));
    common::configure(cfg);
    user::configure(cfg, state.clone());
    bucket::configure(cfg, state.clone());
    user_bucket::configure(cfg, state.clone());
    auth::configure(cfg, state.clone());
}
