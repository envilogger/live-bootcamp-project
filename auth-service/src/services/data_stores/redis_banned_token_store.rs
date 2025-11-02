use std::sync::Arc;
use tokio::sync::RwLock;

use redis::Commands;

use crate::{
    domain::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<redis::Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<redis::Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn ban_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let key: String = get_key(token);

        let mut connection = self.conn.write().await;

        connection
            .set_ex::<String, bool, ()>(key, true, TOKEN_TTL_SECONDS)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let mut connection = self.conn.write().await;

        let has_key = connection
            .exists::<String, bool>(key)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

        Ok(has_key)
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
