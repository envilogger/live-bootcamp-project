use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, Clone)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashMapUserStore {
    users: HashMap<String, User>,
}

impl HashMapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<User, UserStoreError> {
        match self.get_user(email) {
            Ok(user) if user.password == password => Ok(user),
            Ok(_) => Err(UserStoreError::InvalidCredentials),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> HashMapUserStore {
        let mut hm = HashMapUserStore::default();
        let user = User::new(
            "existing@test.tst".to_owned(),
            "password123".to_owned(),
            false,
        );
        let _ = hm.add_user(user);
        hm
    }

    #[tokio::test]
    async fn test_add_user_succeed() {
        let mut store = setup();
        let user = User::new("new@test.tst".to_owned(), "password123".to_owned(), false);
        let result = store.add_user(user);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_user_fail_user_already_exists() {
        let user = User::new(
            "existing@test.tst".to_owned(),
            "password123".to_owned(),
            false,
        );
        let mut store = setup();
        let result = store.add_user(user);
        assert!(matches!(result, Err(UserStoreError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_get_user_returns_existing_user() {
        let store = setup();
        let result = store.get_user("existing@test.tst");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_returns_user_not_found() {
        let store = setup();
        let result = store.get_user("new@test.tst");
        assert!(matches!(result, Err(UserStoreError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_validate_user_succeed_for_valid_parameters() {
        let store = setup();
        let result = store.validate_user("existing@test.tst", "password123");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_fail_for_inexistent_user() {
        let store = setup();
        let result = store.validate_user("new@test.tst", "password123");
        assert!(matches!(result, Err(UserStoreError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_validate_user_fail_for_invalid_password() {
        let store = setup();
        let result = store.validate_user("existing@test.tst", "wrongpassword");
        assert!(matches!(result, Err(UserStoreError::InvalidCredentials)));
    }
}
