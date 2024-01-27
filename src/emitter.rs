use reqwest::Client;
use crate::types::Message;

pub struct Emitter {
    client: Client,
    url: String,
}

impl Emitter {
    pub fn new<S>(url: S) -> Self
        where S: Into<String>
    {
        Self {
            client: Client::new(),
            url: url.into(),
        }
    }

    pub async fn emit(&self, messages: Vec<Message>) -> Result<(), reqwest::Error> {
        self.client.post(&self.url)
            .json(&messages)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;
    use chrono::Utc;

    #[tokio::test]
    async fn test_emit() {
        let emitter = Emitter::new("http://localhost:8000");
        let messages = vec![
            Message::new("test_name".to_string(), "value".to_string(), Utc::now(), None),
            Message::new("test_name".to_string(), "value".to_string(), Utc::now(), None),
        ];

        // should fail
        assert!(emitter.emit(messages).await.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn test_emit_with_server() {
        let emitter = Emitter::new("http://localhost:8000");
        let messages = vec![
            Message::new("test_name".to_string(), "Test Message".to_string(), Utc::now(), Some("1.0".to_string())),
            Message::new("test_name".to_string(), "Test Message".to_string(), Utc::now(), None),
        ];

        // should succeed
        assert!(emitter.emit(messages).await.is_ok());
    }
}