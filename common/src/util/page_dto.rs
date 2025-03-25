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
