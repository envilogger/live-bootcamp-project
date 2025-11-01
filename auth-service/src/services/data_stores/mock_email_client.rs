use crate::domain::EmailClient;

pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recepient: &crate::domain::Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String> {
        println!("Sending email to {} with subject: {} and content: {}", recepient.as_ref(), subject, content);
        Ok(())
    }
}
