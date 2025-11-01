use std::sync::Arc;

use auth_service::{app_state::AppState, utils::constants, Application};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = Arc::new(RwLock::new(
        auth_service::services::HashMapUserStore::default(),
    ));

    let banned_token_store = Arc::new(RwLock::new(
        auth_service::services::HashsetBannedTokenStore::default(),
    ));

    let two_fa_code_store = Arc::new(RwLock::new(
        auth_service::services::HashMapTwoFACodeStore::default(),
    ));

    let email_client = Arc::new(
        auth_service::services::MockEmailClient{},
    );

    let app_state = AppState::new(user_store, banned_token_store, two_fa_code_store, email_client);

    let app = Application::build(app_state, constants::prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
