use uuid::Uuid;

use crate::domain::{Email, Password, User};

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<User, UserStoreError>;
}

#[async_trait::async_trait]
pub trait TwoFACodeStore: Send + Sync {
    async fn add_code(&mut self, email: Email, login_attempt_id: LoginAttemptId, code: TwoFACode) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        Uuid::parse_str(&id)
            .map_err(|e| format!("Invalid UUID format: {}", e))
            .map(|_| LoginAttemptId(id))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let code = format!("{:06}", rand::random::<u32>() % 1_000_000);
        TwoFACode(code)
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TwoFACode(String);
#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn ban_token(&mut self, token: &str);
    async fn is_token_banned(&self, token: &str) -> bool;
}

#[derive(Debug, Clone)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}
