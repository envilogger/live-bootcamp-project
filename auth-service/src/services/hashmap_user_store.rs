use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::domain::{User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashMapUserStore {
    users: Arc<RwLock<HashMap<String, User>>>,
}

#[async_trait::async_trait]
impl UserStore for HashMapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.read().await.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        self.users.write().await.insert(user.email.clone(), user);
        Ok(())
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.read().await.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<User, UserStoreError> {
        match self.get_user(email).await {
            Ok(user) if user.password == password => Ok(user),
            Ok(_) => Err(UserStoreError::InvalidCredentials),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> HashMapUserStore {
        let mut hm = HashMapUserStore::default();
        let user = User::new(
            "existing@test.tst".to_owned(),
            "password123".to_owned(),
            false,
        );
        let _ = hm.add_user(user).await;
        hm
    }

    #[tokio::test]
    async fn test_add_user_succeed() {
        let mut store = setup().await;
        let user = User::new("new@test.tst".to_owned(), "password123".to_owned(), false);
        let result = store.add_user(user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_user_fail_user_already_exists() {
        let user = User::new(
            "existing@test.tst".to_owned(),
            "password123".to_owned(),
            false,
        );
        let mut store = setup().await;
        let result = store.add_user(user).await;
        assert!(matches!(result, Err(UserStoreError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_get_user_returns_existing_user() {
        let store = setup().await;
        let result = store.get_user("existing@test.tst").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_returns_user_not_found() {
        let store = setup().await;
        let result = store.get_user("new@test.tst").await;
        assert!(matches!(result, Err(UserStoreError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_validate_user_succeed_for_valid_parameters() {
        let store = setup().await;
        let result = store
            .validate_user("existing@test.tst", "password123")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_fail_for_inexistent_user() {
        let store = setup().await;
        let result = store.validate_user("new@test.tst", "password123").await;
        assert!(matches!(result, Err(UserStoreError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_validate_user_fail_for_invalid_password() {
        let store = setup().await;
        let result = store
            .validate_user("existing@test.tst", "wrongpassword")
            .await;
        assert!(matches!(result, Err(UserStoreError::InvalidCredentials)));
    }
}
