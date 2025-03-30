use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use strum_macros::{AsRefStr, EnumString};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, Default,PartialEq, Eq, Type, EnumString, AsRefStr,ToSchema)]
pub enum OrderType {
    #[default]
    ASC,
    DESC,
}
#[derive(Debug, Serialize, Deserialize, FromRow,ToSchema,Clone,Default)]
#[serde(rename_all = "camelCase")]
pub struct UserCache {
    pub id: i64,
    pub is_admin: bool,
    pub user_name: String,
    pub bucket_list: Vec<BucketCache>,
}
impl UserCache{

}
#[derive(Debug, Serialize, Deserialize, FromRow,ToSchema,Clone)]
#[serde(rename_all = "camelCase")]
pub struct BucketCache {
    pub bucket_id: i64,
    pub name: String,
    pub right: RightType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type, EnumString, ToSchema)]
#[sqlx(type_name = "ENUM")] // **告诉 `sqlx` 这是 `ENUM` 类型**
#[sqlx(rename_all = "lowercase")]
pub enum RightType {
    Read,
    Write,
    ReadWrite
}
#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema,Clone,IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    #[param(default = 0)]
    pub index: i64,
    #[param(default = 10)]
    pub page_size: i64,
    #[param(default = "id")]
    pub order_column:String,
    pub order_type:OrderType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema,Clone)]
#[serde(rename_all = "camelCase")]
pub struct Page<T>
{
    pub total: i64,
    pub data: Vec<T>,
    pub page_info: PageInfo,
}
