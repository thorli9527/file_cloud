mod handlers;
mod swagger;

use actix_web::{App, HttpServer, cookie, web};
// use app_api::ApiDoc;
use crate::swagger::ApiDoc;
use actix_session::SessionMiddleware;
use actix_session::config::PersistentSession;
use actix_web::cookie::Key;
use actix_web::middleware::Logger;
use common::{AppState, UserCache};
use log::info;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;
use app_console::AuthMiddleware;
use model::db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 读取配置文件
    // 初始化数据库连接池
    let config = AppState::from_env();
    let mut log_builder = AppState::build_log(&config);
    log_builder.init();
    //文件夹缓存
    let dir_create_cache: Arc<Cache<String, String>> = Arc::new(
        Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60 * 24)) // 设置 TTL 60 秒
            .max_capacity(1000) // 最大存储 1000 个键值
            .build(),
    );
    //用户 session
    let session_cache: Arc<Cache<String, UserCache>> = Arc::new(
        Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60 * 24)) // 设置 TTL 60 秒
            .max_capacity(1000) // 最大存储 1000 个键值
            .build(),
    );
    //数据库缓存
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
        session_cache,
    };
    let address_and_port = format!("{}:{}", &config.server.host, &config.server.port);
    info!("Starting server on {}", address_and_port);
    let data = web::Data::new(app_status.clone());
    let pool = Arc::new(db::get_conn(&config.database.url).await);
    let api_doc = ApiDoc::openapi();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(AuthMiddleware { state: data.clone() })
            //配置 orm
            .configure(|cfg| {
                model::db::configure(cfg,pool.clone())
            })
            // 配置 控制器
            .configure(|cfg| {
                handlers::configure(cfg, data.clone());
            })
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone()),
            )
    })
    .keep_alive(actix_web::http::KeepAlive::Timeout(
        std::time::Duration::from_secs(600),
    )) // 允许 10 分钟超时
    .bind(address_and_port)?
    .run()
    .await
}
