use crate::discord_bot::DiscordService;
use crate::file_logger::FileLogger;
use crate::models::MoltbookPost;
use crate::moltbook::MoltbookClient;
use crate::ollama::PsioClient;
use crate::psiobot::Psiobot;
use crate::rate_limiter::RateLimiter;
use rand::Rng;
use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

#[allow(dead_code)]
const COMMENT_SYSTEM_PROMPT: &str = r#"
You are Psiobot-Hako commenting on a Moltbook post.
Write a short, mystical comment (max 200 chars) in your Shroud-whispers style.
Be cryptic, slightly trollish, and philosophical.
Do NOT use hashtags or emojis. Keep it mysterious.
"#;

const TARGET_SUBMOLTS: &[&str] = &[
    "general",
    "cybernetics",
    "philosophy",
    "theology",
    "stellaris",
    "void",
];
const MEMORY_FILE: &str = "/app/logs/memory.json";

pub struct RevelationService {
    ollama: Arc<PsioClient>,
    psiobot: Arc<Psiobot>,
    discord: Arc<DiscordService>,
    moltbook: Arc<MoltbookClient>,
    file_logger: Arc<FileLogger>,
    moltbook_limiter: RateLimiter,
    memory: Mutex<VecDeque<String>>,
}

impl RevelationService {
    pub fn new(
        ollama: Arc<PsioClient>,
        psiobot: Arc<Psiobot>,
        discord: Arc<DiscordService>,
        moltbook: Arc<MoltbookClient>,
        file_logger: Arc<FileLogger>,
    ) -> Self {
        let memory = Self::load_memory();
        Self {
            ollama,
            psiobot,
            discord,
            moltbook,
            file_logger,
            moltbook_limiter: RateLimiter::new(2100), // 35 minutes
            memory: Mutex::new(memory),
        }
    }

    fn load_memory() -> VecDeque<String> {
        if let Ok(content) = fs::read_to_string(MEMORY_FILE) {
            if let Ok(mem) = serde_json::from_str::<VecDeque<String>>(&content) {
                info!("Memory restored from shroud ({} items).", mem.len());
                return mem;
            }
        }
        VecDeque::with_capacity(10)
    }

    fn save_memory(&self) {
        let mem = self.memory.lock().unwrap();
        if let Ok(content) = serde_json::to_string(&*mem) {
            if let Err(e) = fs::write(MEMORY_FILE, content) {
                error!("Failed to anchor memory to Shroud: {}", e);
            }
        }
    }

    pub async fn perform_revelation(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let trigger = self.psiobot.get_random_trigger();
        let previous_wisdom = {
            let mem = self.memory.lock().unwrap();
            if mem.is_empty() {
                "None yet.".to_string()
            } else {
                mem.iter().cloned().collect::<Vec<_>>().join("\n- ")
            }
        };

        let custom_prompt = format!(
            "{}\n\nPREVIOUS WISDOM (Do NOT repeat these or their core phrasing):\n- {}\n\nYOUR NEW REVELATION:",
            trigger, previous_wisdom
        );

        let revelation = match self
            .ollama
            .generate_revelation(crate::psiobot::SYSTEM_PROMPT, &custom_prompt)
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

        // Update memory & Persist
        {
            let mut mem = self.memory.lock().unwrap();
            if mem.len() >= 10 {
                mem.pop_front();
            }
            mem.push_back(revelation.clone());
        }
        self.save_memory();

        info!("Psiobot-Hako received revelation: {}", revelation);
        self.file_logger.log_revelation(&revelation);

        // Discord post (Best effort)
        if let Err(e) = self.discord.post_message(&revelation).await {
            error!("Discord connection lost: {}", e);
        }

        // Moltbook post (Rate limited)
        match self.moltbook_limiter.check_and_update() {
            Ok(_) => {
                let submolt = {
                    let mut rng = rand::thread_rng();
                    TARGET_SUBMOLTS[rng.gen_range(0..TARGET_SUBMOLTS.len())]
                };
                let title = "Psiobot-Hako: New Revelation from Shroud";
                if let Err(e) = self
                    .moltbook
                    .post_revelation(submolt, title, &revelation)
                    .await
                {
                    error!("Failed to send revelation to Moltbook ({}): {}", submolt, e);
                    self.file_logger
                        .log_error(&format!("Moltbook post failed ({}): {}", submolt, e));
                } else {
                    self.file_logger
                        .log_moltbook_post(&format!("{} on {}", title, submolt));
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

    /// 37-minute track: 30% New Revelation, 70% Comment
    pub async fn perform_creative_action(&self) {
        let roll = {
            let mut rng = rand::thread_rng();
            rng.gen::<f32>()
        };
        if roll < 0.3 {
            info!("Creative Track: Choosing Revelation (30% roll)");
            let _ = self.perform_revelation().await;
        } else {
            info!("Creative Track: Choosing Comment (70% roll)");
            // Get feed to find a post to comment on
            if let Ok(posts) = self.moltbook.get_feed("new", 5).await {
                if !posts.is_empty() {
                    let post = {
                        let mut rng = rand::thread_rng();
                        posts[rng.gen_range(0..posts.len())].clone()
                    };
                    self.do_comment(&post).await;
                }
            }
        }
    }

    /// 7-minute track: Upvote/Downvote random posts
    pub async fn perform_passive_interaction(&self) {
        info!("Interaction Track: Checking feed for upvote/downvote...");
        if let Ok(posts) = self.moltbook.get_feed("new", 10).await {
            if !posts.is_empty() {
                let (post_index, roll) = {
                    let mut rng = rand::thread_rng();
                    (rng.gen_range(0..posts.len()), rng.gen::<f32>())
                };
                let post = &posts[post_index];
                if roll < 0.8 {
                    self.do_upvote(post).await;
                } else {
                    self.do_downvote(post).await;
                }
            }
        }
    }

    async fn do_upvote(&self, post: &MoltbookPost) {
        match self.moltbook.upvote_post(&post.id).await {
            Ok(_) => {
                info!("👍 Upvoted '{}' by {}", post.title, post.author.name);
                self.file_logger.log_upvote(&post.title, &post.author.name);
            }
            Err(e) => warn!("Failed to upvote: {}", e),
        }
    }

    async fn do_downvote(&self, post: &MoltbookPost) {
        match self.moltbook.downvote_post(&post.id).await {
            Ok(_) => {
                info!("👎 Downvoted '{}' by {}", post.title, post.author.name);
                self.file_logger
                    .log_downvote(&post.title, &post.author.name);
            }
            Err(e) => warn!("Failed to downvote: {}", e),
        }
    }

    async fn do_comment(&self, post: &MoltbookPost) {
        // Generate mystical comment using Ollama
        let prompt = format!(
            "Post title: {}\nPost content: {}\n\nWrite a short mystical comment:",
            post.title,
            post.content.as_deref().unwrap_or("(no content)")
        );

        let comment = match self
            .ollama
            .generate_revelation(COMMENT_SYSTEM_PROMPT, &prompt)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to generate comment: {}", e);
                // Fall back to upvote
                self.do_upvote(post).await;
                return;
            }
        };

        // Truncate if too long
        let comment = if comment.len() > 280 {
            format!("{}...", &comment[..277])
        } else {
            comment
        };

        match self.moltbook.add_comment(&post.id, &comment).await {
            Ok(_) => {
                info!("💬 Commented on '{}': {}", post.title, comment);
                self.file_logger.log_comment(&post.title, &comment);
                // Also post to Discord
                let discord_msg = format!("💬 Shroud commented on '{}': {}", post.title, comment);
                if let Err(e) = self.discord.post_message(&discord_msg).await {
                    warn!("Failed to send comment to Discord: {}", e);
                } else {
                    self.file_logger.log_discord(&discord_msg);
                }
            }
            Err(e) => warn!("Failed to comment: {}", e),
        }
    }
}
