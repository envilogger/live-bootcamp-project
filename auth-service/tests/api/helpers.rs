use std::sync::Arc;

use auth_service::{
    domain::BannedTokenStore,
    get_postgres_pool, get_redis_client,
    utils::constants::{self, DATABASE_URL},
    Application,
};
use reqwest::cookie::Jar;
use sqlx::{postgres::PgPoolOptions, Executor};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: Arc<RwLock<dyn BannedTokenStore>>,
    pub two_fa_code_store: Arc<RwLock<dyn auth_service::domain::TwoFACodeStore>>,
    pub user_store: Arc<RwLock<dyn auth_service::domain::UserStore>>,
    db_name: String,
    cleaned_up: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgres(&db_name).await;

        let user_store = Arc::new(RwLock::new(auth_service::services::PostgresUserStore::new(
            pg_pool,
        )));

        let shared_redis_conn = configure_redis();
        let shared_redis_conn = Arc::new(RwLock::new(shared_redis_conn));

        let banned_token_store = Arc::new(RwLock::new(
            auth_service::services::RedisBannedTokenStore::new(shared_redis_conn.clone()),
        ));

        let two_fa_code_store = Arc::new(RwLock::new(
            auth_service::services::RedisTwoFaCodeStore::new(shared_redis_conn.clone()),
        ));

        let email_client = Arc::new(auth_service::services::MockEmailClient {});

        let app_state = auth_service::app_state::AppState::new(
            user_store.clone(),
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client,
        );

        let app = Application::build(app_state, constants::test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            user_store,
            db_name,
            cleaned_up: false,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body: serde::Serialize>(&self, body: &Body) -> reqwest::Response {
        self.http_client
            .post(format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body: serde::Serialize>(&self, body: &Body) -> reqwest::Response {
        self.http_client
            .post(format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body: serde::Serialize>(&self, body: &Body) -> reqwest::Response {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body: serde::Serialize>(
        &self,
        body: &Body,
    ) -> reqwest::Response {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn cleanup(mut self) {
        let database_url = DATABASE_URL.to_owned();
        delete_database(&database_url, &self.db_name).await;
        self.cleaned_up = true;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.cleaned_up {
            panic!("TestApp database was not cleaned up. Please call the cleanup method before dropping the TestApp instance.");
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", uuid::Uuid::new_v4())
}

async fn configure_postgres(db_name: &str) -> sqlx::PgPool {
    let database_url = DATABASE_URL.to_owned();
    configure_database(&database_url, db_name).await;

    let postgresql_conn_url = format!("{}/{}", database_url, db_name);

    get_postgres_pool(&postgresql_conn_url)
        .await
        .expect("Failed to create Postgres connection pool");

    let pg_pool = auth_service::get_postgres_pool(&database_url)
        .await
        .expect("Failed to create Postgres connection pool");

    sqlx::migrate!("../migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to run database migrations");

    pg_pool
}

async fn configure_database(database_url: &str, db_name: &str) {
    let connection = PgPoolOptions::new()
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database");

    let db_conn_string = format!("{}/{}", database_url, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("../migrations")
        .run(&connection)
        .await
        .expect("Failed to run database migrations");
}

async fn delete_database(database_url: &str, db_name: &str) {
    let connection = PgPoolOptions::new()
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"DROP DATABASE IF EXISTS "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop database");
}

fn configure_redis() -> redis::Connection {
    get_redis_client(constants::REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
