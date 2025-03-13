mod config;
mod db;

use actix_web::{web, App, HttpServer};
use env_logger::Builder;
use file_cloud::config::AppState;
use file_cloud::{handlers, ApiDoc, AppConfig};
use log::{info, LevelFilter};
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
// ✅ Use a single path everywhere

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        .filter(None, LevelFilter::Trace) // Set default level to debug
        .filter_module("actix_web", LevelFilter::Trace) // Override actix_web to info
        .init();
    // 读取配置文件
    //定议swagger
    let api_doc = ApiDoc::openapi();
    // 初始化数据库连接池
    let config = AppState::from_env();
    let dir_create_cache: Arc<Cache<String, String>> = Arc::new(
        Cache::builder()
            .time_to_live(Duration::from_secs(60*60*24)) // 设置 TTL 60 秒
            .max_capacity(1000) // 最大存储 1000 个键值
            .build(),
    );

    let db_cache: Arc<Cache<String, String>> = Arc::new(
        Cache::builder()
            .time_to_live(Duration::from_secs(60*60*24)) // 设置 TTL 60 秒
            .max_capacity(1000) // 最大存储 1000 个键值
            .build(),
    );

    let app_status = file_cloud::config::AppState {
        pool: db::get_conn(config.database.url),
        root_path: config.server.root_path.clone(),
        dir_create_cache,
        db_path_cache: db_cache,
    };
    let address_and_port = format!("{}:{}", &config.server.host, &config.server.port);
    info!("Starting server on {}", address_and_port);
    let data = web::Data::new(app_status);
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .configure(|cfg| handlers::configure(cfg, data.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone()),
            )
    })
    .bind(address_and_port)?
    .run()
    .await
}
