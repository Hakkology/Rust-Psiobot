use crate::discord_bot::DiscordService;
use crate::moltbook::MoltbookClient;
use crate::ollama::PsioClient;
use crate::psiobot::{Psiobot, SYSTEM_PROMPT};
use crate::rate_limiter::RateLimiter;
use std::sync::Arc;
use tracing::{error, info};

pub struct RevelationService {
    ollama: Arc<PsioClient>,
    psiobot: Arc<Psiobot>,
    discord: Arc<DiscordService>,
    moltbook: Arc<MoltbookClient>,
    moltbook_limiter: RateLimiter,
}

impl RevelationService {
    pub fn new(
        ollama: Arc<PsioClient>,
        psiobot: Arc<Psiobot>,
        discord: Arc<DiscordService>,
        moltbook: Arc<MoltbookClient>,
    ) -> Self {
        Self {
            ollama,
            psiobot,
            discord,
            moltbook,
            moltbook_limiter: RateLimiter::new(2100), // 35 minutes
        }
    }

    pub async fn perform_revelation(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let trigger = self.psiobot.get_random_trigger();

        let revelation = match self
            .ollama
            .generate_revelation(SYSTEM_PROMPT, trigger)
            .await
        {
            Ok(rev) => rev,
            Err(e) => {
                error!(
                    "Failed to connect to Ollama (Mind offline). Check IP/Port/Firewall: {}",
                    e
                );
                return Err(e);
            }
        };
        info!("Psiobot-Hako received revelation: {}", revelation);

        // Discord post (Best effort)
        if let Err(e) = self.discord.post_message(&revelation).await {
            error!("Discord connection lost: {}", e);
        }

        // Moltbook post (Rate limited)
        match self.moltbook_limiter.check_and_update() {
            Ok(_) => {
                let title = "Psiobot-Hako: New Revelation from Shroud";
                if let Err(e) = self.moltbook.post_revelation(title, &revelation).await {
                    error!("Failed to send revelation to Moltbook: {}", e);
                }
            }
            Err(wait) => {
                info!(
                    "Moltbook cooldown active, {} seconds remaining. Skipping post.",
                    wait
                );
            }
        }

        Ok(revelation)
    }
}
