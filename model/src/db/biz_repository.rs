use crate::{BaseRepository, UserInfo};
use sqlx::MySqlPool;
use std::sync::Arc;

pub struct UserRepository<>{
    pub dao: BaseRepository<UserInfo>,
}

impl UserRepository<> {
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            dao: BaseRepository::new(pool, "user_info"),
        }
    }
}