use std::{collections::HashSet, sync::Arc};

use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Default)]
pub struct HashsetBannedTokenStore {
    tokens: Arc<RwLock<HashSet<String>>>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn ban_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.to_owned());
        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.contains(token))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_ban_and_check_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "test_token";

        store.ban_token(token).await.expect("Failed to ban token");

        assert!(store.is_token_banned(token).await.expect("Failed to check token"));
    }

    #[tokio::test]
    async fn test_check_unbanned_token() {
        let store = HashsetBannedTokenStore::default();
        let token = "unbanned_token";

        assert!(!store.is_token_banned(token).await.expect("Failed to check token"));
    }
}
