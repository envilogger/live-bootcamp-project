use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{
    app_state::AppState,
    domain::{error::AuthAPIError, User},
    services::UserStoreError,
};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    if email.is_empty() || !email.contains('@') {
        return Err(AuthAPIError::InvalidCredentials);
    }

    if password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user: User = User::new(email, password, request.requires_2fa);

    let mut user_store = state.user_store.write().unwrap();

    user_store.add_user(user).map_err(|e| match e {
        UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
        _ => AuthAPIError::UnexpectedError,
    })?;

    let response = Json(SignupResponse {
        message: "User created successfully".to_owned(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(serde::Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl From<SignupRequest> for User {
    fn from(val: SignupRequest) -> Self {
        User::new(val.email, val.password, val.requires_2fa)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
