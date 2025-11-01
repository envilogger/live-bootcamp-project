use auth_service::{domain::*, routes::TwoFactorAuthResponse, utils::constants::JWT_COOKIE_NAME};
use uuid::Uuid;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = vec![
        serde_json::json!({ "wrong_field": "not-a-code" }),
        serde_json::json!({}),
    ];

    for case in test_cases {
        let response = app.post_verify_2fa(&case).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let test_cases = vec![
        serde_json::json!({ "email": "not-an-email", "login_attempt_id": "d498ab94-f157-453f-a6e2-da196ae3e713", "two_fa_code": "123456" }),
        serde_json::json!({ "email": "test@example.org", "login_attempt_id": "", "two_fa_code": "123456" }),
        serde_json::json!({ "email": "test@example.org", "login_attempt_id": "valid-id", "two_fa_code": "12" }),
    ];

    for case in test_cases {
        let response = app.post_verify_2fa(&case).await;
        assert_eq!(response.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::parse("123456").unwrap();

    let verify_2fa_body = serde_json::json!({
        "email": email.as_ref() as &str,
        "login_attempt_id": login_attempt_id.as_ref(),
        "two_fa_code": two_fa_code.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_incorrect_code() {
    let app = TestApp::new().await;

    let email = Email::parse("test@example.org".to_owned()).unwrap();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::parse("123456").unwrap();

    app.two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code)
        .await
        .expect("Failed to add 2FA code to store");

    let verify_2fa_body = serde_json::json!({
        "email": email.as_ref(),
        "login_attempt_id": login_attempt_id.as_ref(),
        "two_fa_code": "000000"
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let email = Email::parse(random_email.clone()).unwrap();
    let random_password = Uuid::new_v4().to_string();
    let password = Password::parse(random_password.clone()).unwrap();
    let user = User::new(email.clone(), password.clone(), true);

    // create user
    app.user_store
        .write()
        .await
        .add_user(user)
        .await
        .expect("Failed to create user");

    let login_request = serde_json::json!({
        "email": &random_email,
        "password": &random_password,
    });

    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let login_response = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap();

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .expect("Failed to get 2FA code");

    assert_eq!(
        login_attempt_id.as_ref(),
        login_response.login_attempt_id.as_ref() as &str
    );

    // call login twice, it should replace the old code
    let _ = app.post_login(&login_request).await;

    let verify_2fa_body = serde_json::json!({
        "email": random_email,
        "login_attempt_id": login_response.login_attempt_id,
        "two_fa_code": two_fa_code.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::parse("123456").unwrap();

    app.two_fa_code_store
        .write()
        .await
        .add_code(
            Email::parse(email.clone()).unwrap(),
            login_attempt_id.clone(),
            two_fa_code.clone(),
        )
        .await
        .expect("Failed to add 2FA code to store");

    let verify_2fa_body = serde_json::json!({
        "email": email.as_ref() as &str,
        "login_attempt_id": login_attempt_id.as_ref(),
        "two_fa_code": two_fa_code.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_401_for_same_code_twice() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::parse("123456").unwrap();

    app.two_fa_code_store
        .write()
        .await
        .add_code(
            Email::parse(email.clone()).unwrap(),
            login_attempt_id.clone(),
            two_fa_code.clone(),
        )
        .await
        .expect("Failed to add 2FA code to store");

    let verify_2fa_body = serde_json::json!({
        "email": email.as_ref() as &str,
        "login_attempt_id": login_attempt_id.as_ref(),
        "two_fa_code": two_fa_code.as_ref()
    });

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 401);
}
