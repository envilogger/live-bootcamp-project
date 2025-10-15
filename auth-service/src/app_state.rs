use std::sync::Arc;

use tokio::sync::RwLock;

use crate::domain::UserStore;

#[derive(Clone)]
pub struct AppState {
    pub user_store: Arc<RwLock<dyn UserStore>>,
}

impl AppState {
    pub fn new(user_store: Arc<RwLock<dyn UserStore>>) -> Self {
        Self { user_store }
    }
}
