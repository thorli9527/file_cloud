pub mod models;

use actix_web::web;
pub use models::*;
use sqlx::MySqlPool;
use std::sync::Arc;
pub mod repository;
pub use repository::*;
pub mod biz_repository;
pub mod date_format;
pub use biz_repository::*;
pub use date_format::*;

pub fn configure(cfg: &mut web::ServiceConfig, pool: Arc<MySqlPool>) {
    let user_rep: UserRepository = UserRepository::new(pool.clone());
    let path_rep: PathRepository = PathRepository::new(pool.clone());
    let bucket_rep: BucketRepository = BucketRepository::new(pool.clone());
    let user_bucket_rep: UserBucketRepository = UserBucketRepository::new(pool.clone());
    let user_bucket_right: UserBucketRepository = UserBucketRepository::new(pool.clone());
    let file_rep: FileRepository = FileRepository::new(pool.clone());
    let del_rep  = PathDelTaskRepository::new(pool.clone());
    cfg.app_data(web::Data::new(file_rep));
    cfg.app_data(web::Data::new(user_bucket_rep));
    cfg.app_data(web::Data::new(user_rep));
    cfg.app_data(web::Data::new(user_bucket_right));
    cfg.app_data(web::Data::new(path_rep));
    cfg.app_data(web::Data::new(bucket_rep));
    cfg.app_data(web::Data::new(del_rep));
}
