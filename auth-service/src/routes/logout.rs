use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    domain::error::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = jar
        .get(crate::utils::constants::JWT_COOKIE_NAME)
        .ok_or(AuthAPIError::MissingToken)?;

    validate_token(cookie.value())
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    let jar = jar.remove(JWT_COOKIE_NAME);

    Ok((jar, StatusCode::OK))
}
