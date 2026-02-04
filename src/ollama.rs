use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct PsioClient {
    client: Client,
    endpoint: String,
    model: String,
}

impl PsioClient {
    pub fn new(endpoint: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn generate_revelation(
        &self,
        system_prompt: &str,
        user_input: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: user_input.to_string(),
            stream: false,
            system: system_prompt.to_string(),
        };

        let res = self
            .client
            .post(&format!("{}/api/generate", self.endpoint))
            .json(&request)
            .send()
            .await?;

        let body: OllamaResponse = res.json().await?;
        Ok(body.response)
    }
}
