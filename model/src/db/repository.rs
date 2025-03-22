use async_trait::async_trait;
use common::{AppError, Page, PageInfo, ValueItem};
use sqlx::MySqlPool;
use sqlx::mysql::{MySqlArguments, MySqlRow};
use sqlx::{Arguments, FromRow};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::process::id;
use std::sync::Arc;
use utoipa::openapi::RefOr::T;
pub async  fn query_by_sql<T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow> +Clone+ Send + Sync + Unpin>
(
    pool: Arc<MySqlPool>,
    sql: &str,
    params: HashMap<&str, impl AsRef<str>+ std::marker::Send>,
) -> Result<Vec<T>, AppError> {
    let mut sql_query = sqlx::query_as::<_, T>(sql);
    for (key, value) in &params {
        sql_query = sql_query.bind(value.as_ref());
    }
    Ok(sql_query.fetch_all( &*pool).await?)
}
/// 定义 Repository Trait，所有 Repository 都要实现这些方法
#[async_trait]
pub trait Repository<T: for<'r> sqlx::FromRow<'r, MySqlRow>> {
    //query all
    async fn get_all(&self) -> Result<Vec<T>, AppError>;
    //find by id
    async fn find_by_id(&self, id: impl AsRef<str>+ std::marker::Send) -> Result<T, AppError>;
    //delete by id
    async fn del_by_id(&self, id: impl AsRef<str>+ std::marker::Send) -> Result<u64, AppError>;
    //query by params
    async fn query_by_params(&self, params: HashMap<&str, String>) -> Result<Vec<T>, AppError>;
    //query count
    async fn query_by_count(&self, params: HashMap<&str, String>) -> Result<i64, AppError>;
    //page query
    async fn query_by_page(
        &self,
        params: HashMap<&str, String>,
        page_info: &PageInfo,
    ) -> Result<Page<T>, AppError>;
    //query one
    async fn find_by_one(&self, params: HashMap<&str, String>) -> Result<T, AppError>;
    //insert
    async fn insert(&self, params: HashMap<&str, String>) -> Result<u64, AppError>;
    //change data by id
    async fn change(&self, id: &String, params: HashMap<&str, String>) -> Result<(), AppError>;
}

/// 泛型 BaseRepository，支持所有表
///
#[allow(dead_code)]
pub struct BaseRepository<T> {
    pub pool: Arc<MySqlPool>, // 线程安全的数据库连接池
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
    T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow> + Send + Sync + Unpin+ Clone, // 需要实现 `FromRow`
{
    async fn get_all(&self) -> Result<Vec<T>, AppError> {
        let query = format!("SELECT * FROM {}", self.table_name);
        let vec = sqlx::query_as::<_, T>(&query)
            .fetch_all(&*self.pool)
            .await?;
        return Ok(vec);
    }

    async fn find_by_id(&self, id: impl AsRef<str>+ std::marker::Send) -> Result<T, AppError> {
        let query = format!("SELECT * FROM {} WHERE id = ?", self.table_name);
        let option = sqlx::query_as::<_, T>(&query)
            .bind(id.as_ref())
            .fetch_one(&*self.pool)
            .await?;
        return Ok(option);
    }

    async fn del_by_id(&self, id: impl AsRef<str>+ std::marker::Send) -> Result<u64, AppError> {
        let query = format!("DELETE FROM {} WHERE id = ?", self.table_name);
        let result = sqlx::query(&query).bind(id.as_ref()).execute(&*self.pool).await?;
        Ok(result.rows_affected())
    }

    async fn find_by_one(&self, params: HashMap<&str, String>) -> Result<T, AppError> {
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
        let exit_error_option = params.get("exit_error");
        match exit_error_option {
            Some(exit_error) => {
                if exit_error == "true" {
                    return Ok(sql_query.fetch_one(&*self.pool).await?)
                }
            }
            None => {
                let list=sql_query.fetch_all(&*self.pool).await?;
                if(list.len()> 0){
                    return Ok(list[0].clone());
                }
                return Err(AppError::NotFound("Not Found".to_string()));
            }
        }
        let result = sql_query.fetch_one(&*self.pool).await?;
        Ok(result)
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

        let result = sql_query.fetch_all(&*self.pool).await?;
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
        let result = sql_query.execute(&*self.pool).await?;
        Ok(result.rows_affected())
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
        sqlx::query_with(&query, args).execute(&*self.pool).await?;
        Ok(())
    }

    async fn query_by_count(&self, params: HashMap<&str, String>) -> Result<i64, AppError> {
        let mut query = format!("SELECT count(1) FROM {} WHERE ", self.table_name);
        let mut values = vec![];

        for (key, value) in &params {
            query.push_str(&format!("{} = ? AND ", key));
            values.push(value.clone());
        }
        if &params.len()> &0 {
            // 移除最后的 "AND "
            query.truncate(query.len() - 4);
        }
        else{
            query.truncate(query.len() - 7);
        }


        let mut sql_query = sqlx::query_scalar(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }

        let result = sql_query.fetch_one(&*self.pool).await?;
        Ok(result)
    }

    async fn query_by_page(
        &self,
        params: HashMap<&str, String>,
        page_info: &PageInfo,
    ) -> Result<Page<T>, AppError> {
        let mut  offset = match page_info.index{
            index=>index+1
        }  * page_info.page_size;
        let mut limit=page_info.page_size;
       let count=self.query_by_count(params.clone()).await?;
        if(count<page_info.page_size){
            offset=0;
            limit=page_info.page_size;
        }


        let mut query = format!("SELECT * FROM {} WHERE ", self.table_name);
        let mut values = vec![];

        for (key, value) in &params {
            query.push_str(&format!("{} = ? AND ", key));
            values.push(value.clone());
        }
        if &params.len()> &0 {
            // 移除最后的 "AND "
            query.truncate(query.len() - 4);
        }
        else{
            query.truncate(query.len() - 7);
        }

        let order_type=match &page_info.order_type {
            (order_type) => order_type.as_ref().to_string(),
            _ => "ASC".to_string(),
        };
        let order_str = &format!(" ORDER BY {} {} LIMIT {} OFFSET {}", page_info.order_column, order_type, limit, offset);
        query.push_str(order_str);
        let mut sql_query = sqlx::query_as::<_, T>(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }

        let list = sql_query.fetch_all(&*self.pool).await?;
        Ok(Page {
            total: count,
            data: list,
            page_info: PageInfo{
                index: page_info.index,
                page_size: page_info.page_size,
                total: count,
                order_column: page_info.order_column.clone(),
                order_type: page_info.order_type.clone(),
            },
        })
    }
}
