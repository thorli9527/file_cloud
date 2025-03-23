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

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>) {
    common::configure(cfg);
    upload::configure(cfg, state.clone());
}
