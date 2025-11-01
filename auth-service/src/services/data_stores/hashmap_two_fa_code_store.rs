use std::collections::HashMap;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

#[derive(Default)]
pub struct HashMapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashMapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), crate::domain::TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), crate::domain::TwoFACodeStoreError> {
        self.codes.remove(email);
        Ok(())
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some((login_attempt_id, code)) => Ok((login_attempt_id.clone(), code.clone())),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain::Email, services::HashMapTwoFACodeStore};

    #[tokio::test]
    async fn should_add_and_get_code() {
        use crate::domain::TwoFACodeStore;

        let mut store = HashMapTwoFACodeStore::default();
        let email = Email::parse("test@example.org".to_owned()).unwrap();
        let login_attempt_id = crate::domain::LoginAttemptId::default();
        let code = crate::domain::TwoFACode::default();

        store.add_code(email.clone(), login_attempt_id.clone(), code.clone()).await.unwrap();

        let (stored_attempt_id, stored_code) = store.get_code(&email).await.unwrap();
        assert_eq!(stored_attempt_id, login_attempt_id);
        assert_eq!(stored_code, code);

        store.remove_code(&email).await.unwrap();
        let result = store.get_code(&email).await;

        assert!(result.is_err());
    }
}
