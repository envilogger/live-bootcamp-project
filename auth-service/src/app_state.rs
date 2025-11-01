use std::sync::Arc;

use tokio::sync::RwLock;

use crate::domain::{BannedTokenStore, UserStore};

#[derive(Clone)]
pub struct AppState {
    pub user_store: Arc<RwLock<dyn UserStore>>,
    pub banned_token_store: Arc<RwLock<dyn crate::domain::BannedTokenStore>>,
    pub two_fa_code_store: Arc<RwLock<dyn crate::domain::TwoFACodeStore>>,
}

impl AppState {
    pub fn new(
        user_store: Arc<RwLock<dyn UserStore>>,
        banned_token_store: Arc<RwLock<dyn BannedTokenStore>>,
        two_fa_code_store: Arc<RwLock<dyn crate::domain::TwoFACodeStore>>,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
        }
    }
}
