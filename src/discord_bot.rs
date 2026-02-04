use serenity::all::{Http, ChannelId, CreateMessage};
use std::sync::Arc;

pub struct DiscordService {
    http: Arc<Http>,
    channel_id: ChannelId,
}

impl DiscordService {
    pub fn new(token: &str, channel_id: u64) -> Self {
        Self {
            http: Arc::new(Http::new(token)),
            channel_id: ChannelId::new(channel_id),
        }
    }

    pub async fn post_message(&self, content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let builder = CreateMessage::new().content(content);
        self.channel_id.send_message(&self.http, builder).await?;
        Ok(())
    }
}
