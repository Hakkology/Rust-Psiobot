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
    pub id: Option<String>,
    pub post: Option<MoltbookPost>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub retry_after_minutes: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct MoltbookAuthor {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct MoltbookSubmolt {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct MoltbookPost {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub upvotes: i32,
    #[serde(default)]
    pub downvotes: i32,
    pub author: MoltbookAuthor,
    pub submolt: Option<MoltbookSubmolt>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct MoltbookFeedResponse {
    pub success: bool,
    #[serde(default)]
    pub posts: Vec<MoltbookPost>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct MoltbookCommentRequest {
    pub content: String,
}

// API Models
#[derive(Serialize)]
pub struct RevelationResponse {
    pub message: String,
    pub status: String,
}
