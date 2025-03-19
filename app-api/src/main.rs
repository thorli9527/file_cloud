mod handlers;

use actix_web::{web, App, HttpServer};
// use app_api::ApiDoc;
use common::AppState;
use log::info;
use model::db;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
// use model::UserRepository::UserRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 读取配置文件
    //定议swagger
    // let api_doc = ApiDoc::openapi();
    // 初始化数据库连接池
    let config = AppState::from_env();
    let mut log_builder = AppState::build_log(&config);
    log_builder.init();
    let dir_create_cache: Arc<Cache<String, String>> = Arc::new(
        Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60 * 24)) // 设置 TTL 60 秒
            .max_capacity(1000) // 最大存储 1000 个键值
            .build(),
    );

    let db_cache: Arc<Cache<String, String>> = Arc::new(
        Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60 * 24)) // 设置 TTL 60 秒
            .max_capacity(1000) // 最大存储 1000 个键值
            .build(),
    );

    let app_status = AppState {
        root_path: config.server.root_path.clone(),
        dir_create_cache,
        db_path_cache: db_cache,
    };
    let address_and_port = format!("{}:{}", &config.server.host, &config.server.port);
    info!("Starting server on {}", address_and_port);
    let data = web::Data::new(app_status.clone());
    let pool=Arc::new(db::get_conn(&config.database.url).await);
    HttpServer::new(move || {
        App::new()
            .configure(|cfg| {
                handlers::configure(cfg, data.clone(),pool.clone());
            })
    })
    .keep_alive(actix_web::http::KeepAlive::Timeout(
        std::time::Duration::from_secs(600),
    )) // 允许 10 分钟超时
    .bind(address_and_port)?
    .run()
    .await
}
