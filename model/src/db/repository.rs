use async_trait::async_trait;
use common::AppError;
use sqlx::mysql::MySqlArguments;
use sqlx::{Arguments, FromRow};
use sqlx::{Database, MySqlPool};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
/// 定义 Repository Trait，所有 Repository 都要实现这些方法
#[async_trait]
pub trait Repository<T> {
    async fn get_all(&self) -> Result<Vec<T>, AppError>;
    async fn find_by_id(&self, id: String) -> Result<Option<T>, AppError>;
    async fn del_by_id(&self, id: String) -> Result<u64, AppError>;
    async fn query_by_params(&self, params: HashMap<&str, String>) -> Result<Vec<T>, AppError>;
    async fn insert(&self, params: HashMap<&str, String>) -> Result<u64, AppError>;
    async fn change(&self, id: &String, params: HashMap<&str, String>) -> Result<(), AppError>;
}

/// 泛型 BaseRepository，支持所有表
///
#[allow(dead_code)]
pub struct BaseRepository<T> {
    pool: Arc<MySqlPool>, // 线程安全的数据库连接池
    pub table_name: &'static str,
    _marker: PhantomData<T>,
}

impl<T> BaseRepository<T> {
    pub fn new(pool: Arc<MySqlPool>, table_name: &'static str) -> Self {
        Self {
            pool,
            table_name,
            _marker: Default::default(),
        }
    }
}

#[async_trait]
impl<T> Repository<T> for BaseRepository<T>
where
    T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow> + Send + Sync + Unpin, // 需要实现 `FromRow`
{
    async fn get_all(&self) -> Result<Vec<T>, AppError> {
        let query = format!("SELECT * FROM {}", self.table_name);
        let vec = sqlx::query_as::<_, T>(&query).fetch_all(&*self.pool).await.unwrap();
        return Ok(vec);
    }

    async fn find_by_id(&self, id:String) -> Result<Option<T>, AppError> {
        let query = format!("SELECT * FROM {} WHERE id = ?", self.table_name);
        let option = sqlx::query_as::<_, T>(&query)
            .bind(id)
            .fetch_optional(&*self.pool)
            .await.unwrap();
        return Ok(option);
    }

    async fn del_by_id(&self, id: String) -> Result<u64, AppError> {
        let query = format!("DELETE FROM {} WHERE id = ?", self.table_name);
        let result = sqlx::query(&query).bind(id).execute(&*self.pool).await.unwrap();
        Ok(result.rows_affected())
    }

    async fn query_by_params(&self, params: HashMap<&str, String>) -> Result<Vec<T>, AppError> {
        let mut query = format!("SELECT * FROM {} WHERE ", self.table_name);
        let mut values = vec![];

        for (key, value) in &params {
            query.push_str(&format!("{} = ? AND ", key));
            values.push(value.clone());
        }
        // 移除最后的 "AND "
        query.truncate(query.len() - 4);

        let mut sql_query = sqlx::query_as::<_, T>(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }

        let result = sql_query.fetch_all(&*self.pool).await.unwrap();
        Ok(result)
    }

    async fn insert(&self, params: HashMap<&str, String>) -> Result<u64, AppError> {
        let keys: Vec<&str> = params.keys().cloned().collect();
        let values: Vec<String> = params.values().cloned().collect();

        let placeholders = vec!["?"; keys.len()].join(", ");
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name,
            keys.join(", "),
            placeholders
        );

        let mut sql_query = sqlx::query(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }
        let result = sql_query.execute(&*self.pool).await.unwrap();
        Ok((result.rows_affected()))
    }
    async fn change(&self, id: &String, params: HashMap<&str, String>) -> Result<(), AppError> {
        let mut query = String::from("UPDATE ");
        query.push_str(&*self.table_name);
        query.push_str(" SET ");
        let mut args = MySqlArguments::default();
        let mut first = true;
        for (key, value) in &params {
            if !first {
                query.push(',');
            }
            first = false;
            query.push_str(&format!(" {} = ?", key));
            args.add(value);
        }
        query.push_str(" WHERE id = ?");
        args.add(id);
        sqlx::query_with(&query, args).execute(&*self.pool).await;
        Ok(())
    }
}
