use auth_service::{domain::Email, utils::auth::generate_auth_cookie};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({}),
        serde_json::json!({
            "not-a-token": "invalid_token"
        }),
    ];

    for body in test_cases {
        let response = app.post_verify_token(&body).await;
        assert_eq!(response.status().as_u16(), 422);
    }

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_200_for_valid_token() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let email = Email::parse(email).unwrap();
    let token = generate_auth_cookie(&email).unwrap().value().to_owned();

    let body = serde_json::json!({
        "token": token
    });

    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 200);

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_for_incorrect_token() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "token": "invalid_token"
    });

    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 401);

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_for_banned_token() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let email = Email::parse(email).unwrap();
    let token = generate_auth_cookie(&email).unwrap().value().to_owned();

    app.banned_token_store.write().await.ban_token(&token).await;

    let body = serde_json::json!({
        "token": token
    });

    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 401);

    app.cleanup().await;
}
