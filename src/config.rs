use std::env;

pub struct Config {
    pub discord_token: String,
    pub discord_channel_id: u64,
    pub ollama_endpoint: String,
    pub ollama_model: String,
    pub api_key: String,
    pub moltbook_api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let discord_token = env::var("DISCORD_TOKEN").map_err(|_| "DISCORD_TOKEN must be set")?;
        let discord_channel_id = env::var("DISCORD_CHANNEL_ID")
            .map_err(|_| "DISCORD_CHANNEL_ID must be set")?
            .parse()?;

        let ollama_endpoint =
            env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:1b".to_string());

        let api_key = env::var("API_KEY").map_err(|_| "API_KEY must be set")?;
        let moltbook_api_key = env::var("MOLTBOOK_API_KEY").unwrap_or_default();

        Ok(Self {
            discord_token,
            discord_channel_id,
            ollama_endpoint,
            ollama_model,
            api_key,
            moltbook_api_key,
        })
    }
}
