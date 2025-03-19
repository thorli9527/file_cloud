pub mod common;
mod download;
pub mod upload;
pub use upload::*;
pub mod user;
pub use user::*;
use user::*;
pub mod bucket;
pub use bucket::*;
mod auth;
mod swagger;
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use ::common::AppState;
use actix_web::web;
use model::{Bucket, BucketRepository, PathInfo, PathRepository, UserBucket, UserBucketRepository, UserInfo};
use model::biz_repository::UserRepository;
use sqlx::MySqlPool;
use std::sync::Arc;
use actix_web::web::ServiceConfig;

pub fn configure(cfg: &mut web::ServiceConfig, state: web::Data<AppState>, pool: Arc<MySqlPool>)  {
    let user_info:UserRepository = UserRepository::new(pool.clone());
    let path_info:PathRepository = PathRepository::new(pool.clone());
    let bucket_rep:BucketRepository = BucketRepository::new(pool.clone());
    let user_bucket_rep:UserBucketRepository = UserBucketRepository::new(pool.clone());
    let user_bucket_right:UserBucketRepository = UserBucketRepository::new(pool.clone());
    cfg.app_data(web::Data::new(user_info));
    cfg.app_data(web::Data::new(path_info));
    cfg.app_data(web::Data::new(bucket_rep));
    cfg.app_data(web::Data::new(user_bucket_rep));
    cfg.app_data(web::Data::new(user_bucket_right));
    common::configure(cfg);
    user::configure(cfg, state.clone());
    let api_doc = ApiDoc::openapi();
    cfg.service(user::user_list);
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone()),
    );
}
