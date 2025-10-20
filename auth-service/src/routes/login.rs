use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{
    app_state::AppState,
    domain::{error::AuthAPIError, Email, Password, UserStoreError},
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let (email, password) = (request.email, request.password);

    match login_int(&state, email, password).await {
        Ok(auth_token) => (jar.add(auth_token), Ok(StatusCode::OK)),
        Err(e) => (jar, Err(e)),
    }
}

async fn login_int(
    app_state: &AppState,
    email: String,
    password: String,
) -> Result<Cookie<'static>, AuthAPIError> {
    let email = Email::parse(email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password = Password::parse(password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = app_state.user_store.read().await;

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

    Ok(auth_cookie)
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
