use crate::models::{OllamaRequest, OllamaResponse};
use reqwest::Client;

pub struct PsioClient {
    client: Client,
    endpoint: String,
    model: String,
}

impl PsioClient {
    pub fn new(endpoint: &str, model: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            endpoint: endpoint.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn generate_revelation(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/generate", self.endpoint);
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            system: system.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;
        let body: OllamaResponse = response.json().await?;

        Ok(body.response)
    }
}
