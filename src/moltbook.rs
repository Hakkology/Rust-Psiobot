use crate::models::{
    MoltbookCommentRequest, MoltbookFeedResponse, MoltbookPost, MoltbookPostRequest,
    MoltbookPostResponse,
};
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
        submolt: &str,
        title: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.api_key.is_empty() {
            return Err("Moltbook API key is missing".into());
        }

        let url = format!("{}/posts", self.base_url);
        let request = MoltbookPostRequest {
            submolt: submolt.to_string(),
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

        let status = response.status();
        let body_text = response.text().await?;

        if status.is_success() {
            if let Ok(body) = serde_json::from_str::<MoltbookPostResponse>(&body_text) {
                let post_id = body
                    .id
                    .or_else(|| body.post.as_ref().map(|p| p.id.clone()))
                    .unwrap_or_else(|| "unknown".to_string());
                tracing::info!(
                    "Revelation successfully posted to Moltbook! ID: {} 🦞",
                    post_id
                );
            } else {
                tracing::info!(
                    "Revelation successfully posted to Moltbook! (Response body: {}) 🦞",
                    body_text
                );
            }
            Ok(())
        } else {
            tracing::error!("Moltbook API Error ({}) - Response: {}", status, body_text);

            if let Ok(body) = serde_json::from_str::<MoltbookPostResponse>(&body_text) {
                let error_msg = body.error.unwrap_or_else(|| "Unknown error".to_string());
                if status.as_u16() == 429 {
                    let retry = body.retry_after_minutes.unwrap_or(30);
                    tracing::warn!(
                        "Moltbook Rate Limit: Retry in {} minutes. Error: {}",
                        retry,
                        error_msg
                    );
                }
                Err(format!("Moltbook API error: {} - {}", status, error_msg).into())
            } else {
                Err(format!("Moltbook API error: {} - {}", status, body_text).into())
            }
        }
    }

    /// Get feed posts from Moltbook
    pub async fn get_feed(
        &self,
        sort: &str,
        limit: u32,
    ) -> Result<Vec<MoltbookPost>, Box<dyn std::error::Error + Send + Sync>> {
        if self.api_key.is_empty() {
            return Err("Moltbook API key is missing".into());
        }

        let url = format!("{}/posts?sort={}&limit={}", self.base_url, sort, limit);

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            let body: MoltbookFeedResponse = response.json().await?;
            Ok(body.posts)
        } else {
            let status = response.status();
            Err(format!("Failed to get Moltbook feed: {}", status).into())
        }
    }

    /// Upvote a post
    pub async fn upvote_post(
        &self,
        post_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.api_key.is_empty() {
            return Err("Moltbook API key is missing".into());
        }

        let url = format!("{}/posts/{}/upvote", self.base_url, post_id);

        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Upvoted post: {}", post_id);
            Ok(())
        } else {
            let status = response.status();
            Err(format!("Failed to upvote: {}", status).into())
        }
    }

    /// Downvote a post
    pub async fn downvote_post(
        &self,
        post_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.api_key.is_empty() {
            return Err("Moltbook API key is missing".into());
        }

        let url = format!("{}/posts/{}/downvote", self.base_url, post_id);

        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("Downvoted post: {}", post_id);
            Ok(())
        } else {
            let status = response.status();
            Err(format!("Failed to downvote: {}", status).into())
        }
    }

    /// Add a comment to a post
    pub async fn add_comment(
        &self,
        post_id: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.api_key.is_empty() {
            return Err("Moltbook API key is missing".into());
        }

        let url = format!("{}/posts/{}/comments", self.base_url, post_id);
        let request = MoltbookCommentRequest {
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
            tracing::info!("Comment added to post: {}", post_id);
            Ok(())
        } else {
            let status = response.status();
            Err(format!("Failed to add comment: {}", status).into())
        }
    }
}
