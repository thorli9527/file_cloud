use crate::{BaseRepository, UserInfo, Bucket, FileInfo, PathInfo, UserBucket, Repository, UserBucketRight};
use sqlx::{MySqlPool};
use std::{str, sync::Arc};
use std::collections::HashMap;
use common::AppError;

pub struct UserRepository{
    pub dao: BaseRepository<UserInfo>,
}

impl UserRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_info"),
        }
    }
    pub async fn find_by_access_key(&self,  access_key: String) -> Result<UserInfo,AppError> {
        let mut params:HashMap<&str, String> = HashMap::new();
        params.insert("access_key", access_key.to_string());
        return  self.dao.find_by_one(params).await;
    }
}


pub struct PathRepository{
    pub dao: BaseRepository<PathInfo>,
}

impl PathRepository{
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "path_info"),
        }
    }
}

pub struct BucketRepository{
    pub dao: BaseRepository<Bucket>,
}

impl BucketRepository{
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "bucket"),
        }
    }
    pub async fn find_by_name(&self, name: String) -> Result<Bucket,AppError> {
        let mut params:HashMap<&str, String> = HashMap::new();
        params.insert("name", name.to_string());
        return  self.dao.find_by_one(params).await;
    }
}

pub struct FileRepository{
    pub dao: BaseRepository<FileInfo>,
}

impl FileRepository{
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "file_info"),
        }
    }
}

pub struct UserBucketRepository{
    pub dao: BaseRepository<UserBucket>,
}

impl UserBucketRepository{
    pub fn new(pool:Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_bucket"),
        }
    }
}

pub struct UserBucketRightRepository{
    pub dao: BaseRepository<UserBucketRight>,
}

impl UserBucketRightRepository{
    pub fn new(pool:Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_bucket_right"),
        }
    }
}

