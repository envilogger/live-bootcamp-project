pub mod app_state;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

use crate::{app_state::AppState, domain::error::AuthAPIError, utils::tracing::*};
use axum::{
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::post,
    serve::Serve,
    Json, Router,
};
use redis::RedisResult;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::error::Error;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

const PG_POOL_MAX_CONNECTIONS: u32 = 5;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = ["http://localhost:8000".parse()?];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/logout", post(routes::logout))
            .route("/verify-token", post(routes::verify_token))
            .with_state(app_state)
            .layer(cors)
            .layer(TraceLayer::new_for_http()
              .make_span_with(make_span_with_request_id)
              .on_request(on_request)
              .on_response(on_response));

        let listener = tokio::net::TcpListener::bind(address).await.unwrap();
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("listening on {}", &self.address);
        self.server.await
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => {
                (StatusCode::UNAUTHORIZED, "Incorrect credentials")
            }
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::Invalid2FACodeRequest => (StatusCode::BAD_REQUEST, "Invalid 2FA code"),
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}

pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().max_connections(PG_POOL_MAX_CONNECTIONS).connect(url).await
}

pub fn get_redis_client(redis_hostname: String) -> RedisResult<redis::Client> {
  let redis_url = format!("redis://{}/", redis_hostname);
  redis::Client::open(redis_url)
}
