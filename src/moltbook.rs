use crate::models::{MoltbookPostRequest, MoltbookPostResponse};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};

pub struct MoltbookClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl MoltbookClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            base_url: "https://www.moltbook.com/api/v1".to_string(),
        }
    }

    pub async fn post_revelation(
        &self,
        title: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.api_key.is_empty() {
            return Err("Moltbook API key is missing".into());
        }

        let url = format!("{}/posts", self.base_url);
        let request = MoltbookPostRequest {
            submolt: "general".to_string(),
            title: title.to_string(),
            content: content.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Moltbook'a vahiy başarıyla iletildi! 🦞");
            Ok(())
        } else {
            let status = response.status();
            let body: MoltbookPostResponse = response.json().await?;
            let error_msg = body.error.unwrap_or_else(|| "Unknown error".to_string());

            if status.as_u16() == 429 {
                let retry = body.retry_after_minutes.unwrap_or(30);
                tracing::warn!(
                    "Moltbook Rate Limit: {} dakika sonra tekrar dene. Hata: {}",
                    retry,
                    error_msg
                );
            } else {
                tracing::error!("Moltbook Hatası ({}): {}", status, error_msg);
            }

            Err(format!("Moltbook API error: {} - {}", status, error_msg).into())
        }
    }
}
