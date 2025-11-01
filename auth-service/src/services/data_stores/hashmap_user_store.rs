use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

#[derive(Default)]
pub struct HashMapUserStore {
    users: Arc<RwLock<HashMap<Email, User>>>,
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

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.read().await.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<User, UserStoreError> {
        match self.get_user(email).await {
            Ok(user) if user.password == *password => Ok(user),
            Ok(_) => Err(UserStoreError::InvalidCredentials),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: create a struct for predefined users to avoid repetition and possible mistakes
    use super::*;

    fn get_valid_email(i: u8) -> Email {
        Email::parse(format!("existing{}@test.txt", i).to_owned()).unwrap()
    }

    fn get_valid_password() -> Password {
        Password::parse("password123".to_owned()).unwrap()
    }

    async fn setup() -> HashMapUserStore {
        let mut hm = HashMapUserStore::default();
        let user = User::new(get_valid_email(1), get_valid_password(), false);
        let _ = hm.add_user(user).await;
        hm
    }

    #[tokio::test]
    async fn test_add_user_succeed() {
        let mut store = setup().await;
        let user = User::new(get_valid_email(2), get_valid_password(), false);
        let result = store.add_user(user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_user_fail_user_already_exists() {
        let user = User::new(get_valid_email(1), get_valid_password(), false);
        let mut store = setup().await;
        let result = store.add_user(user).await;
        assert!(matches!(result, Err(UserStoreError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_get_user_returns_existing_user() {
        let store = setup().await;
        let result = store.get_user(&get_valid_email(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_returns_user_not_found() {
        let store = setup().await;
        let result = store.get_user(&get_valid_email(3)).await;
        assert!(matches!(result, Err(UserStoreError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_validate_user_succeed_for_valid_parameters() {
        let store = setup().await;
        let result = store
            .validate_user(&get_valid_email(1), &get_valid_password())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_fail_for_inexistent_user() {
        let store = setup().await;
        let result = store
            .validate_user(&get_valid_email(3), &get_valid_password())
            .await;
        assert!(matches!(result, Err(UserStoreError::UserNotFound)));
    }

    #[tokio::test]
    async fn test_validate_user_fail_for_invalid_password() {
        let store = setup().await;
        let result = store
            .validate_user(
                &get_valid_email(1),
                &Password::parse("wrongpassword".to_owned()).unwrap(),
            )
            .await;
        assert!(matches!(result, Err(UserStoreError::InvalidCredentials)));
    }
}
