use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{app_state::AppState, domain::User};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    let mut user_store = state.user_store.write().unwrap();
    user_store.add_user(request.into()).unwrap();

    let response = Json(SignupResponse {
        message: "User created successfully".to_owned(),
    });

    (StatusCode::CREATED, response)
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
