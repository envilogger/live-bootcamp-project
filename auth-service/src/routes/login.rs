use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{error::AuthAPIError, Email, Password, UserStoreError},
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password =
        Password::parse(request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = state.user_store.read().await;

    user_store
        .validate_user(&email, &password)
        .await
        .map_err(|e| match e {
            UserStoreError::UserNotFound => AuthAPIError::IncorrectCredentials,
            UserStoreError::InvalidCredentials => AuthAPIError::IncorrectCredentials,
            _ => AuthAPIError::UnexpectedError,
        })?;

    let user = user_store
        .get_user(&email)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let auth_cookie = crate::utils::auth::generate_auth_cookie(&user.email)
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let jar = jar.add(auth_cookie);
    Ok((jar, StatusCode::OK))
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
