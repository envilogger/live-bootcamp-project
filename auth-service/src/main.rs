use std::sync::Arc;

use auth_service::{
    Application, app_state::AppState, get_postgres_pool, services::PostgresUserStore, utils::constants::{self, DATABASE_URL}
};
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgres().await;

    let user_store = Arc::new(RwLock::new(
        PostgresUserStore::new(pg_pool),
    ));

    let banned_token_store = Arc::new(RwLock::new(
        auth_service::services::HashsetBannedTokenStore::default(),
    ));

    let two_fa_code_store = Arc::new(RwLock::new(
        auth_service::services::HashMapTwoFACodeStore::default(),
    ));

    let email_client = Arc::new(auth_service::services::MockEmailClient {});

    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let app = Application::build(app_state, constants::prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgres() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool");

    sqlx::migrate!("../migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to run database migrations");

    pg_pool
}
