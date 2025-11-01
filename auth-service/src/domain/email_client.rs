use crate::domain::Email;

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(
        &self,
        recepient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String>;
}
