use std::sync::Arc;

use auth_service::{app_state::AppState, utils::constants, Application};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = auth_service::services::HashMapUserStore::default();
    let app_state = AppState::new(Arc::new(RwLock::new(user_store)));

    let app = Application::build(app_state, constants::prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
