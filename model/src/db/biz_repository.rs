use crate::{query_by_sql, BaseRepository, Bucket, FileInfo, FileItemDto, PathDelTask, PathInfo, QueryParam, Repository, UserBucket, UserBucketRight, UserInfo};
use common::{build_md5, build_snow_id, build_time, AppError};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, MySql, MySqlPool, Transaction};
use std::collections::HashMap;
use std::{str, sync::Arc};
use chrono::Local;
use validator::Validate;


pub struct PathDelTaskRepository {
    pub dao: BaseRepository<PathDelTask>,
}
impl PathDelTaskRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "path_del_task"),
        }
    }
    pub async fn create(&self, task: PathDelTask, path_rep: &PathRepository) -> Result<(), AppError> {
        let mut tx: Transaction<'_, MySql> = self.dao.pool.begin().await?;
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("id", task.id.to_string());
        params.insert("path_id", task.path_id.to_string());
        params.insert("del_file_status", "0".to_owned());
        params.insert("del_path_status", "0".to_owned());
        params.insert("create_time", build_time().await.to_owned());
        self.dao.insert(params).await?;
        path_rep.dao.del_by_id(task.path_id).await?;
        tx.commit().await?;
        Ok(())
    }
}

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
        return self.dao.find_by_one(vec![QueryParam::eq("user_name", user_name.as_str())]).await;
    }
    pub async fn login(&self, user_name: &String, password: &String) -> Result<UserInfo, AppError> {
        let user_result = self.dao.query_by_params(vec![QueryParam::eq("user_name", user_name.as_str())]).await?;
        if user_result.len() > 0 {
            let info = &user_result[0];
            if info.password == build_md5(password) {
                return Ok(info.clone());
            }
        }
        Err(AppError::BizError("username.or.password.error".to_string()))
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

    pub async fn new_path(&self, path: &String, pid: &i64, bucket_id: &i64) -> Result<i64, AppError> {
        let mut params: HashMap<&str, String> = HashMap::new();
        let mut full_path = String::new();
        if pid != &0 {
            let parent_info = self.dao.find_by_id(pid.clone()).await?;
            full_path = format!("{}/{}", parent_info.path, &path);
            params.insert("root", "0".to_owned());
        } else {
            params.insert("root", "1".to_owned());
            full_path = path.clone();
        }
        params.insert("bucket_id", bucket_id.to_string());
        params.insert("path", path.to_string());
        let i = build_snow_id();
        params.insert("id", i.to_string());
        params.insert("parent", pid.to_string());
        params.insert("full_path", full_path);
        params.insert("create_time", build_time().await);
        self.dao.insert(params).await?;
        return Ok(i);
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
    pub async fn find_by_name(&self, name: &String) -> Result<Bucket, AppError> {
        return self.dao.find_by_one(vec![QueryParam::eq("name", name.to_string().as_str())]).await;
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
    pub async fn insert(
        &self,
        params: HashMap<&str, String>,
        items: &Vec<FileItemDto>,
    ) -> Result<(), AppError> {
        let mut keys: Vec<&str> = params.keys().cloned().collect();
        let values: Vec<String> = params.values().cloned().collect();
        keys.push("items");
        let placeholders = vec!["?"; keys.len()].join(", ");
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.dao.table_name,
            keys.join(", "),
            placeholders
        );

        let mut sql_query = sqlx::query(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }
        sql_query = sql_query.bind(Json(items));
        sql_query.execute(&*self.dao.pool.clone()).await?;
        Ok(())
    }


    pub async fn path_size(&self, full_path: &str) -> Result<i64, AppError> {
        let query = format!(
            r#"
           SELECT
                COALESCE(SUM(file_info.size), 0)
            FROM
                file_info
            WHERE
                file_info.full_path LIKE '{}%'
                "#,
            full_path
        );
        let mut sql_query = sqlx::query_scalar::<_, Decimal>(&query);

        let result = sql_query.fetch_one(&*self.dao.pool).await?;
        Ok(result.to_i64().unwrap())
    }


    pub async fn path_file_list(&self, full_path: &str, max_id: i64, bucket_id: i64) -> Result<Vec<FileInfo>, AppError> {
        let query = format!(
            r#"
            SELECT * from {} where bucket_id ={} and full_path LIKE '{}/%' and id>{} order by id asc
                "#,
            self.dao.table_name, bucket_id, full_path, max_id
        );
        let list_result = sqlx::query_as::<_, FileInfo>(&query)
            .fetch_all(&*self.dao.pool)
            .await?;
        return Ok(list_result);
    }
}
#[derive(Debug, Serialize, Deserialize, FromRow, Validate, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BucketInfoResult {
    pub id:i64,
    pub bucket_id: i64,
    pub user_id: i64,
    pub user_name: String,
    //0 读 1 写 2 读写
    pub user_right: i32,
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

    pub async fn change_right(
        &self,
        user_id: i64,
        bucket_id: i64,
        right: i32,
    ) -> Result<(), AppError> {
        let list = self.dao.query_by_params(vec![
            QueryParam::eq("user_id", user_id.to_string().as_str()),
            QueryParam::eq("bucket_id", bucket_id.to_string().as_str())
        ]).await?;
        if (list.len() == 1) {
            let mut params: HashMap<&str, String> = HashMap::new();
            params.insert("user_right",right.to_string(),);
            self.dao.change(list[0].id, params).await?;
            return Ok(());
        } else {
            let mut params: HashMap<&str, String> = HashMap::new();
            params.insert("id", build_snow_id().to_string());
            params.insert("user_id", user_id.to_string());
            params.insert("bucket_id", bucket_id.to_string());
            params.insert("user_right",right.to_string(),);
            self.dao.insert(params).await?;
        }
        Ok(())
    }

    pub async fn query_by_user_id_and_bucket_Id(&self, user_id: &i64, bucket_id: &i64) -> Result<Vec<BucketInfoResult>, AppError> {
        let params: HashMap<&str, String> = HashMap::new();
        let sql = format!(r#"
            SELECT distinct
                user_bucket.id,
                bucket.id as bucket_id,
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
                user_bucket.user_id = {} and bucket_id={}
                "#, user_id, bucket_id);
        let vec = query_by_sql::<BucketInfoResult>(self.dao.pool.clone(), &sql, vec![]).await?;
        Ok(vec)
    }

    pub async fn query_by_bucket_id(
        &self,
        bucket_id: &i64,
    ) -> Result<Vec<BucketInfoResult>, AppError> {
        if bucket_id == &0 {
            return Err(AppError::InvalidInput("InvalidInput bucketId".to_owned()));
        }
        let sql = format!(r#"
         SELECT
            user_bucket.bucket_id,
            user_info.user_name,
            user_info.id as user_id,
            user_bucket.id,
            user_bucket.user_right
        FROM
            user_bucket
            INNER JOIN
            user_info
            ON
                user_bucket.user_id = user_info.id
        WHERE
            user_bucket.bucket_id = {}
                "#, bucket_id);
        let vec = query_by_sql::<BucketInfoResult>(self.dao.pool.clone(), &sql, vec![]).await?;
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
