use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{BannedTokenStoreError, error::AuthAPIError},
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = jar
        .get(crate::utils::constants::JWT_COOKIE_NAME)
        .ok_or(AuthAPIError::MissingToken)?;

    let token = cookie.value().to_owned();

    let mut banned_token_store = app_state.banned_token_store.write().await;

    validate_token(&token, &*banned_token_store)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    let jar = jar.remove(JWT_COOKIE_NAME);

    banned_token_store.ban_token(&token).await.map_err(|e| match e {
        BannedTokenStoreError::UnexpectedError => {
            AuthAPIError::UnexpectedError
        }
    })?;

    Ok((jar, StatusCode::OK))
}
