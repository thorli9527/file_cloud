use config::Config;
use env_logger::Builder;
use log::LevelFilter;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use moka::future::Cache;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logs: LogsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct LogsConfig {
    //全局日志级别
    pub global: String,
    //error 模块:级别,
    pub error: String,
    //warn 模块:级别,
    pub warn: String,
    //warn 模块:级别,
    pub info: String,
    //debug 模块:级别,
    pub debug: String,
    //trace 模块:级别,
    pub trace: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub root_path: String,
}
#[derive(Clone)]
pub struct AppState {
    pub root_path: String,
    pub dir_create_cache: Arc<Cache<String, String>>,
    pub db_path_cache: Arc<Cache<String, String>>,
    pub session_cache: Arc<Cache<String, String>>,
}

impl AppState {
    pub fn from_env() -> AppConfig {
        let config = Config::builder()
            .add_source(config::File::with_name("file-cloud.toml").required(true))
            .add_source(config::Environment::with_prefix("APP").separator("_"))
            .build()
            .expect("Failed to build configuration");

        config
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize configuration")
    }
    pub fn build_log(config: &AppConfig) -> Builder {
        let mut builder = Builder::new();

        let logs = &config.logs;
        builder.filter(None, LevelFilter::from_str(&logs.global).unwrap());
        if !logs.warn.is_empty() {
            for cfg in logs.warn.split(",") {
                builder.filter_module(cfg, LevelFilter::Trace);
            }
        }
        if !logs.error.is_empty() {
            for cfg in logs.error.split(",") {
                builder.filter_module(cfg, LevelFilter::Error);
            }
        }
        if !logs.info.is_empty() {
            for cfg in logs.info.split(",") {
                builder.filter_module(cfg, LevelFilter::Info);
            }
        }
        if !logs.debug.is_empty() {
            for cfg in logs.debug.split(",") {
                builder.filter_module(cfg, LevelFilter::Debug);
            }
        }
        if !logs.trace.is_empty() {
            for cfg in logs.trace.split(",") {
                builder.filter_module(cfg, LevelFilter::Trace);
            }
        }

        builder
    }
}
