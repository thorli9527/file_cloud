use serde::Deserialize;
use std::fs;
use std::path::Path;
use config::Config;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use crate::db;

#[derive(Debug, Deserialize,Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize,Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize,Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub root_path:String
}
#[derive(Clone)]
pub struct AppState {
    pub pool: mysql::Pool,
    pub root_path: String,
    pub dir_create_cache:Arc<Cache<String, String>> ,
    pub db_path_cache:Arc<Cache<String, String>>,
}

impl AppState {
    pub fn from_env() -> AppConfig {
        let mut config = Config::builder()
            .add_source(config::File::with_name("file-cloud.toml").required(true))
            .add_source(config::Environment::with_prefix("APP").separator("_"))
            .build()
            .expect("Failed to build configuration");

        config
            .try_deserialize::<AppConfig>()
            .expect("Failed to deserialize configuration")
    }
}