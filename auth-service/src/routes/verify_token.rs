use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::{domain::error::AuthAPIError, utils::auth::validate_token};

pub async fn verify_token(
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    validate_token(&request.token).map_err(|_| AuthAPIError::InvalidToken)?;

    Ok(StatusCode::OK.into_response())
}

#[derive(serde::Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
