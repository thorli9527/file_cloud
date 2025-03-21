use crate::{
    BaseRepository, Bucket, FileInfo, PathInfo, Repository, RightType, UserBucket, UserBucketRight,
    UserBucketRightQueryResult, UserInfo,
};
use common::{AppError, build_md5};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};
use std::collections::HashMap;
use std::{str, sync::Arc};
use utoipa::ToSchema;
use validator::Validate;

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
    pub async fn login(&self, user_name: String, password: String) -> Result<UserInfo, AppError> {
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
#[derive(Debug, Serialize, Deserialize, FromRow,Validate)]
pub struct BucketInfoResult {
    name: String,
    right: RightType,
}
impl UserBucketRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_bucket"),
        }
    }

    pub async fn find_by_user_name(
        &self,
        user_name: String,
    ) -> Result<Vec<BucketInfoResult>, AppError> {
        if user_name.is_empty() {
            return Err(AppError::BizError("user_name.is_empty".to_owned()));
        }
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("user_name", user_name);
        let sql = r#"
            SELECT distinct
                user_bucket.`right`,
                bucket.`name`
            FROM
                user_bucket
                INNER JOIN
                user_info
                ON
                    user_bucket.user_id = user_info.id
                INNER JOIN
                bucket
                ON
                    user_bucket.bucket_id = bucket.id
            WHERE
                user_info.user_name = ?")"#;
        let vec = self
            .dao
            .query_by_sql::<BucketInfoResult>(sql.to_string(), params)
            .await?;
        Ok(vec)
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
