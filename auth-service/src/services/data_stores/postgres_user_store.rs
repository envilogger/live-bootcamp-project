use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use sqlx::PgPool;
use tokio::task::spawn_blocking;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = spawn_blocking(async move || -> Result<String, UserStoreError> {
            compute_password_hash(user.password.as_ref().to_string())
                .await
                .map_err(|_| UserStoreError::UnexpectedError)
        })
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?
        .await?;

        sqlx::query!(
            r#"INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3);"#,
            user.email.as_ref() as &str,
            password_hash.as_ref() as &str,
            user.requires_2fa,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            e.into_database_error()
                .and_then(|db_err| {
                    if db_err.kind() == sqlx::error::ErrorKind::UniqueViolation {
                        Some(UserStoreError::UserAlreadyExists)
                    } else {
                        Some(UserStoreError::UnexpectedError)
                    }
                })
                .unwrap_or(UserStoreError::UnexpectedError)
        })?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let record = sqlx::query!(
            r#"SELECT email, password_hash, requires_2fa FROM users WHERE email = $1;"#,
            email.as_ref() as &str
        );

        let record = record
            .fetch_one(&self.pool)
            .await
            .map_err(|_| UserStoreError::UserNotFound)?;

        // TODO: Storing password hash is meaningless. Need to refactor the code.
        //       password should not be part of User at all.

        let user = User {
            email: Email::parse(record.email).unwrap(),
            password: Password::parse(record.password_hash).unwrap(),
            requires_2fa: record.requires_2fa,
        };

        Ok(user)
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<User, UserStoreError> {
        let user = self.get_user(email).await?;

        spawn_blocking({
            let expected_password_hash = user.password.as_ref().to_string();
            let password_candidate = password.as_ref().to_string();

            async move || -> Result<(), UserStoreError> {
                verify_password_hash(expected_password_hash, password_candidate)
                    .await
                    .map_err(|_| UserStoreError::InvalidCredentials)
            }
        })
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?
        .await?;

        Ok(user)
    }
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt = SaltString::generate(&mut rand::thread_rng());

            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
    })
    .await?
}

#[tracing::instrument(name = "Verifying password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_hash)
                .map_err(|e| e.into())
        })
    })
    .await?
}
