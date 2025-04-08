use async_trait::async_trait;
use common::{AppError, OrderType, Page, PageInfo};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use sqlx::MySqlPool;
use sqlx::{Arguments, FromRow};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
pub async fn query_by_sql<T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow> + Clone + Send + Sync + Unpin>
(
    pool: Arc<MySqlPool>,
    sql: &str,
    params: Vec<QueryParam>,
) -> Result<Vec<T>, AppError> {
    let (where_clause, values) = build_where_clause(&params);
    // let mut sql_query = sqlx::query_as::<_, T>(sql);
    // for value in values {
    //     sql_query = sql_query.bind(value);
    // }
    //
    // let result = sql_query.fetch_all(pool).await?;
    // Ok(result)





    let mut sql_query = sqlx::query_as::<_, T>(&sql);
    for value in values {
        sql_query = sql_query.bind(value);
    }

    let result = sql_query.fetch_all(&*pool).await?;
    Ok(result)
}
/// 定义 Repository Trait，所有 Repository 都要实现这些方法
#[async_trait]
pub trait Repository<T: for<'r> sqlx::FromRow<'r, MySqlRow>> {
    //query all
    async fn get_all(&self) -> Result<Vec<T>, AppError>;
    //find by id
    async fn find_by_id(&self, id: i64) -> Result<T, AppError>;
    //delete by id
    async fn del_by_id(&self, id: i64) -> Result<u64, AppError>;
    //query by params
    async fn query_by_params(&self,  params: Vec<QueryParam>,) -> Result<Vec<T>, AppError>;
    //query count
    async fn query_by_count(&self, params: Vec<QueryParam>,) -> Result<i64, AppError>;
    //query by sql
    async fn query_by_sql(&self, sql: &String) -> Result<Vec<T>, AppError>;
    //page query
    async fn query_by_page(
        &self,
        params: Vec<QueryParam>,
        page_info: &PageInfo,
    ) -> Result<Page<T>, AppError>;
    async fn query_by_max_id(
        &self,
        id: i64,
        params: Vec<QueryParam>,
        order_type: OrderType,
        page_size: &i16,
    ) -> Result<Vec<T>, AppError>;
    //query one
    async fn find_by_one(&self,  params: Vec<QueryParam>,) -> Result<T, AppError>;
    //insert
    async fn insert(&self,   params: HashMap<&str,String>,) -> Result<u64, AppError>;
    //change data by id
    async fn change(&self, id: i64,  params: HashMap<&str,String>,) -> Result<(), AppError>;
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
    T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow> + Send + Sync + Unpin + Clone, // 需要实现 `FromRow`
{
    async fn get_all(&self) -> Result<Vec<T>, AppError> {
        let query = format!("SELECT * FROM {}", self.table_name);
        let vec = sqlx::query_as::<_, T>(&query)
            .fetch_all(&*self.pool)
            .await?;
        return Ok(vec);
    }


    async fn find_by_id(&self, id: i64) -> Result<T, AppError> {
        let query = format!("SELECT * FROM {} WHERE id = {}", self.table_name, id);
        let option = sqlx::query_as::<_, T>(&query)
            .fetch_one(&*self.pool)
            .await?;
        return Ok(option);
    }

    async fn del_by_id(&self, id: i64) -> Result<u64, AppError> {
        let query = format!("DELETE FROM {} WHERE id = {}", self.table_name, id);
        let result = sqlx::query(&query).bind(id).execute(&*self.pool).await?;
        Ok(result.rows_affected())
    }

    async fn find_by_one(&self, params: Vec<QueryParam>) -> Result<T, AppError> {
        let (where_clause, values) = build_where_clause(&params);
        let mut query = format!("SELECT * FROM {}{} ", self.table_name,where_clause);
        let mut sql_query = sqlx::query_as::<_, T>(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }

        let result = sql_query.fetch_all(&*self.pool).await?;
        if result.is_empty(){
            return Err(AppError::NotFound("NotFound".to_string()));
        }
        if result.len()>1{
            return Err(AppError::InternalError("MoreThanOne".to_string()));
        }
        Ok(result[0].clone())
    }
    async fn query_by_sql(&self, sql: &String) -> Result<Vec<T>, AppError> {
        let sql_query = sqlx::query_as::<_, T>(sql);
        let result = sql_query.fetch_all(&*self.pool).await?;
        Ok(result)
    }
    async fn query_by_params(&self, params: Vec<QueryParam>) -> Result<Vec<T>, AppError> {


        let (where_clause, values) = build_where_clause(&params);
        let mut query = format!("SELECT * FROM {}{} ", self.table_name,where_clause);
        let mut sql_query = sqlx::query_as::<_, T>(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }

        let result = sql_query.fetch_all(&*self.pool).await?;
        Ok(result)
    }

    async fn insert(&self, params: HashMap<&str,String>) -> Result<u64, AppError> {
        let keys: Vec<&str> = params.keys().cloned().collect();
        let placeholders = vec!["?"; keys.len()].join(", ");
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name,
            keys.join(", "),
            placeholders
        );

        let mut sql_query = sqlx::query(&query);
        for key_item in keys {
            let value = params.get(key_item).unwrap();
            sql_query = sql_query.bind(value);
            println!(" → Param[{}],value:{}", key_item, value);
        }
        println!(" → sql:{}",&query);
        let result = sql_query.execute(&*self.pool).await?;
        Ok(result.rows_affected())
    }
    async fn change(&self, id: i64,  params: HashMap<&str,String>) -> Result<(), AppError> {
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

    async fn query_by_count(
        &self,
        params: Vec<QueryParam>,
    ) -> Result<i64, AppError> {
        let (where_clause, values) = build_where_clause(&params);
        let sql = format!("SELECT COUNT(1) FROM {}{}", self.table_name, where_clause);

        let mut query = sqlx::query_scalar::<_, i64>(&sql);
        for val in values {
            query = query.bind(val);
        }

        let count = query.fetch_one(&*self.pool).await?;
        Ok(count)
    }

    async fn query_by_max_id(&self, id: i64, params: Vec<QueryParam>, order_type: OrderType,
                             page_size: &i16) -> Result<Vec<T>, AppError> {

        let mut max_params=params.clone();
        max_params.push(QueryParam::gt("id", id.to_string().as_str()));
        let (where_clause, values) = build_where_clause(&max_params);
        let mut query = format!("SELECT * FROM  {}{}", self.table_name, where_clause);

        let order_type = match order_type {
            (order_type) => order_type.as_ref().to_string(),
            _ => "ASC".to_string(),
        };
        let order_str = &format!(" ORDER BY id {} LIMIT {} ", order_type, page_size);
        query.push_str(order_str);
        let mut sql_query = sqlx::query_as::<_, T>(&query);
        for value in values {
            sql_query = sql_query.bind(value);
        }
        let vec1 = sql_query.fetch_all(&*self.pool).await?;
        return Ok(vec1);
    }
    async fn query_by_page(
        &self,
        params: Vec<QueryParam>,
        page_info: &PageInfo,
    ) -> Result<Page<T>, AppError> {
        let mut offset = match page_info.index {
            index => index + 1
        } * page_info.page_size;
        let mut limit = page_info.page_size;
        let count = self.query_by_count(params.clone()).await?;
        if (count < page_info.page_size) {
            offset = 0;
            limit = page_info.page_size;
        }
        let (where_clause, values) = build_where_clause(&params);
        let mut query = format!("SELECT * FROM {}{}", self.table_name, where_clause);

        let order_type = match &page_info.order_type {
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
            page_info: PageInfo {
                index: page_info.index,
                page_size: page_info.page_size,
                order_column: page_info.order_column.clone(),
                order_type: page_info.order_type.clone(),
            },
        })
    }
}
#[derive(Debug)]
enum PropertyType {
    Text,
    Number,
    Date,
}
#[derive(Debug, Eq, PartialEq,Clone)]
pub enum QueryType {
    Equal,
    GreaterThan,
    LessThan,
    Between,
    LikeEnd, // like '%xxx'
}
#[derive(Debug,Clone)]
pub struct QueryParam {
    pub field: String,
    pub query_type: QueryType,
    pub values: Vec<String>,
}
impl QueryParam {
    pub fn eq(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            query_type: QueryType::Equal,
            values: vec![value.to_string()],
        }
    }

    pub fn gt(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            query_type: QueryType::GreaterThan,
            values: vec![value.to_string()],
        }
    }

    pub fn lt(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            query_type: QueryType::LessThan,
            values: vec![value.to_string()],
        }
    }

    pub fn between(field: &str, from: &str, to: &str) -> Self {
        Self {
            field: field.to_string(),
            query_type: QueryType::Between,
            values: vec![from.to_string(), to.to_string()],
        }
    }

    pub fn like_end(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            query_type: QueryType::LikeEnd,
            values: vec![value.to_string()],
        }
    }

    pub fn is_empty(&self) -> bool {
        if self.field.is_empty() {
            return true;
        }

        if self.values.len() == 0 {
            return true;
        }
        if self.query_type == QueryType::Between {
            if self.values.len() != 2 {
                return true;
            }
            if self.query_type == QueryType::LessThan || self.query_type == QueryType::GreaterThan {
                if self.values.len() != 1 {
                    return true;
                }
            }
        }
        if self.values.len()==1{
            if self.query_type== QueryType::LikeEnd && self.query_type==QueryType::Equal{
                if self.values[0].is_empty(){
                    return true;
                }
            }
        }
        return false;
    }
}
fn build_where_clause(params: &[QueryParam]) -> (String, Vec<String>) {
    let mut clauses = Vec::new();
    let mut values = Vec::new();

    for param in params {
        if param.is_empty() {
            continue; // 🧼 跳过空值条件
        }
        let field = &param.field;
        match param.query_type {
            QueryType::Equal => {
                clauses.push(format!("{} = ?", field));
                values.push(param.values[0].clone());
            }
            QueryType::GreaterThan => {
                clauses.push(format!("{} > ?", field));
                values.push(param.values[0].clone());
            }
            QueryType::LessThan => {
                clauses.push(format!("{} < ?", field));
                values.push(param.values[0].clone());
            }
            QueryType::Between => {
                clauses.push(format!("{} BETWEEN ? AND ?", field));
                values.push(param.values[0].clone());
                values.push(param.values[1].clone());
            }
            QueryType::LikeEnd => {
                clauses.push(format!("{} LIKE ?", field));
                values.push(format!("{}%", param.values[0]));
            }
        }
    }

    let where_sql = if clauses.is_empty() {
        "".to_string()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };

    (where_sql, values)
}
