use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{app_state::AppState, domain::error::AuthAPIError, utils::auth::validate_token};

pub async fn verify_token(
    State(app_state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let banned_token_store = app_state.banned_token_store.read().await;

    validate_token(&request.token, &*banned_token_store)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    Ok(StatusCode::OK.into_response())
}

#[derive(serde::Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
