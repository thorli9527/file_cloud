pub mod common;
mod download;
pub mod upload;

use ::common::AppState;
use actix_web::web;
use model::biz_repository::UserRepository;
use model::{
    Bucket, BucketRepository, PathInfo, PathRepository, UserBucket, UserBucketRepository, UserInfo,
};
use sqlx::MySqlPool;
use std::sync::Arc;

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
    upload::configure(cfg, state.clone());
}
