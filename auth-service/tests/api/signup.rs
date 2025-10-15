use auth_service::routes::SignupResponse;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // TODO add more test cases
    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": "true"
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": "true"
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123"
        }),
        serde_json::json!({}),
        serde_json::json!({
            "email": 111,
            "password": "password123",
            "requires2FA": "true"
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_for_valid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let expected_response = SignupResponse {
        message: "User created successfully".to_owned(),
    };

    let response = app.post_signup(&body).await;
    assert_eq!(response.status().as_u16(), 201);
    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Failed to deserialize response body to SignupResponse"),
        expected_response
    );
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // TODO add more test cases
    let test_cases = [
        serde_json::json!({
            "email": "not-an-email",
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": "short",
            "requires2FA": true
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    // Create the user for the first time
    let body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });
    let _ = app.post_signup(&body).await;

    // Try to create the user again
    let response = app.post_signup(&body).await;
    assert_eq!(response.status().as_u16(), 409);
}
