use crate::helpers::TestApp;

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
