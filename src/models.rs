use serde::{Deserialize, Serialize};

// Ollama Models
#[derive(Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub system: String,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

// Moltbook Models
#[derive(Serialize)]
pub struct MoltbookPostRequest {
    pub submolt: String,
    pub title: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct MoltbookPostResponse {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub retry_after_minutes: Option<u32>,
}

// API Models
#[derive(Serialize)]
pub struct RevelationResponse {
    pub message: String,
    pub status: String,
}
