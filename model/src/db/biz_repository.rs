use crate::{
    BaseRepository, Bucket, FileInfo, PathInfo, Repository, UserBucket, UserBucketRight, UserInfo,
};
use common::{build_md5, AppError};
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::{str, sync::Arc};

pub struct UserRepository {
    pub dao: BaseRepository<UserInfo>,
}

impl UserRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_info"),
        }
    }
    pub async fn find_by_name(&self, user_name: String) -> Result<UserInfo, AppError> {
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("user_name", user_name.to_string());
        return self.dao.find_by_one(params).await;
    }
    pub async fn login(&self,user_name:String,password:String) -> Result<UserInfo, AppError> {
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("user_name", user_name.to_string());
        let user_result = self.dao.find_by_one(params).await;
        let user_info = match user_result {
            Ok(info) => info,
            Err(e) => return Err(AppError::NotErrorNoRight(e.to_string())),
        };
        if user_info.password != build_md5(&password) {
            return Err(AppError::NotErrorNoRight("password.error".to_owned()));
        }
        Ok(user_info)
    }
}

pub struct PathRepository {
    pub dao: BaseRepository<PathInfo>,
}

impl PathRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "path_info"),
        }
    }
}

pub struct BucketRepository {
    pub dao: BaseRepository<Bucket>,
}

impl BucketRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "bucket"),
        }
    }
    pub async fn find_by_name(&self, name: String) -> Result<Bucket, AppError> {
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("name", name.to_string());
        return self.dao.find_by_one(params).await;
    }
}

pub struct FileRepository {
    pub dao: BaseRepository<FileInfo>,
}

impl FileRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "file_info"),
        }
    }
}

pub struct UserBucketRepository {
    pub dao: BaseRepository<UserBucket>,
}

impl UserBucketRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_bucket"),
        }
    }
}

pub struct UserBucketRightRepository {
    pub dao: BaseRepository<UserBucketRight>,
}

impl UserBucketRightRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_bucket_right"),
        }
    }
}
