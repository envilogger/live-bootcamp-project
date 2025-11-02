use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_jwt_cookie_invalid() {
    let app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 401);

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    // TODO: instead of sign-up, directly add user to the store
    let _ = app.post_signup(&body).await;

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.post_login(&body).await;

    let cookie = response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("JWT cookie not found during login");

    let token = cookie.value().to_owned();

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    // Verify that the token is banned
    let is_banned = {
        let store = app.banned_token_store.read().await;
        store.is_token_banned(&token).await
    };

    assert!(is_banned, "Token should be banned after logout");

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let _ = app.post_signup(&body).await;

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });
    let _ = app.post_login(&body).await;

    let _ = app.post_logout().await;
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);

    app.cleanup().await;
}
