use auth_service::{
    domain::{Email, LoginAttemptId},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
};

use crate::helpers::{get_random_email, TestApp};

// TODO: use stores directly to set up preconditions instead of going through the API

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let test_cases = vec![
        serde_json::json!({ "wrong_field": "not-an-email", "password": "validpassword123" }),
        serde_json::json!({}),
    ];

    for case in test_cases {
        let response = app.post_login(&case).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let test_cases = vec![
        serde_json::json!({ "email": "invalid-email", "password": "validpassword123" }),
        serde_json::json!({ "email": "validemail@example.com", "password": "short" }),
    ];

    for case in test_cases {
        let response = app.post_login(&case).await;
        assert_eq!(response.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn should_return_401_if_credentials_are_incorrect() {
    let app = TestApp::new().await;

    let login_body = serde_json::json!({
        "email": "validemail@example.com",
        "password": "wrongpassword"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).unwrap();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let response = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Failed to parse response");

    assert_eq!(response.message, "2FA required");

    let attempt_id =
        LoginAttemptId::parse(response.login_attempt_id).expect("Failed to parse login attempt ID");

    let (stored_attempt_id, _) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .expect("2FA code not found");

    assert_eq!(attempt_id, stored_attempt_id);
}
