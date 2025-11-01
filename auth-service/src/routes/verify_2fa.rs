use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{app_state::AppState, domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStoreError, error::AuthAPIError}};

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::Invalid2FACodeRequest)?;
    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id).map_err(|_| AuthAPIError::Invalid2FACodeRequest)?;
    let two_fa_code = TwoFACode::parse(&request.two_fa_code).map_err(|_| AuthAPIError::Invalid2FACodeRequest)?;

    let mut store = state.two_fa_code_store.write().await;

    let (stored_login_attempt_id, stored_code) = store.get_code(&email).await.map_err(|e| match e {
        TwoFACodeStoreError::LoginAttemptIdNotFound => AuthAPIError::IncorrectCredentials,
        TwoFACodeStoreError::UnexpectedError => AuthAPIError::UnexpectedError,
    })?;

    if stored_login_attempt_id != login_attempt_id || stored_code != two_fa_code {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    store.remove_code(&email).await.map_err(|_| AuthAPIError::UnexpectedError)?;

    let auth_cookie = crate::utils::auth::generate_auth_cookie(&email)
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let jar = jar.add(auth_cookie);

    Ok((StatusCode::OK, (jar, Json(()))))
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}
