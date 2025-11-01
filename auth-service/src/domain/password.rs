#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Password(String);

impl Password {
    pub fn parse(s: String) -> Result<Password, String> {
        if s.len() >= 8 {
            Ok(Password(s))
        } else {
            Err("Password must be at least 8 characters long".to_string())
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
