use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use strum_macros::{AsRefStr, EnumString};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Default,PartialEq, Eq, Type, EnumString, AsRefStr,ToSchema)]
pub enum OrderType {
    #[default]
    ASC,
    DESC,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Default, ToSchema,Clone)]
pub struct PageInfo {
    pub index: i64,
    pub page_size: i64,
    pub total: i64,
    pub order_column:String,
    pub order_type:OrderType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema,Clone)]
pub struct Page<T>
{
    pub total: i64,
    pub data: Vec<T>,
    pub page_info: PageInfo,
}
