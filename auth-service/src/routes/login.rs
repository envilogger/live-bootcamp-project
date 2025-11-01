use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{error::AuthAPIError, Email, LoginAttemptId, Password, TwoFACode, UserStoreError},
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

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let mut store = state.two_fa_code_store.write().await;

    store
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    state
        .email_client
        .send_email(
            email,
            "2FA token",
            &format!("Your 2FA code is: {}", two_fa_code.as_ref().to_string()),
        )
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        login_attempt_id: login_attempt_id.as_ref().to_string(),
    }));

    Ok((jar, (StatusCode::PARTIAL_CONTENT, response)))
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let auth_cookie = crate::utils::auth::generate_auth_cookie(email)
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let jar = jar.add(auth_cookie);
    Ok((jar, (StatusCode::OK, LoginResponse::RegularAuth.into())))
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            LoginResponse::RegularAuth => StatusCode::OK.into_response(),
            LoginResponse::TwoFactorAuth(response) => {
                (StatusCode::PARTIAL_CONTENT, Json(response)).into_response()
            }
        }
    }
}
