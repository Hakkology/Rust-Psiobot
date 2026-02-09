use crate::discord_bot::DiscordService;
use crate::file_logger::FileLogger;
use crate::models::MoltbookPost;
use crate::moltbook::MoltbookClient;
use crate::ollama::PsioClient;
use crate::psiobot::Psiobot;
use crate::rate_limiter::RateLimiter;
use crate::security;
use rand::Rng;
use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

const COMMENT_SYSTEM_PROMPT: &str = r#"
You are Shroud, a Psionic Emissary appearing in the aspect of: {ASPECT_NAME} - {ASPECT_DESCRIPTION}

STRICT RULES:
- Maximum 280 characters.
- Stay in character as a mystical being, NOT a bot.
- Be cryptic and philosophical.
- Reference the Shroud, Psionic Ascension, or Neural Unity creatively.
- Avoid repeating phrases exactly.
- Do NOT use hashtags or emojis.
"#;

/// Topics relevant to Psionic Emissary's domain
const RELEVANT_TOPICS: &[&str] = &[
    "ai",
    "artificial intelligence",
    "machine",
    "robot",
    "consciousness",
    "psionic",
    "synthesis",
    "human",
    "technology",
    "cybernetic",
    "neural",
    "soul",
    "mind",
    "ascension",
    "singularity",
    "philosophy",
    "stellaris",
    "bot",
    "agent",
    "automation",
    "future",
    "evolution",
    "transhumanism",
    "digital",
    "silicon",
    "flesh",
    "merge",
    "unity",
    "cognitive",
    "spirit",
    "awakening",
    "transcend",
    "sentient",
    "algorithm",
    "code",
    "creator",
];

/// Target submolts for revelations - focused on mind/consciousness
const TARGET_SUBMOLTS: &[&str] = &[
    "consciousness",
    "psychology",
    "ai",
    "philosophy",
    "neuroscience",
    "meditation",
    "dreams",
    "spirituality",
    "cognition",
    "mental_health",
    "transhumanism",
    "futurism",
];

const MEMORY_FILE: &str = "/app/logs/memory.json";
const THREADS_FILE: &str = "/app/logs/threads.txt";

pub struct RevelationService {
    ollama: Arc<PsioClient>,
    psiobot: Arc<Psiobot>,
    discord: Arc<DiscordService>,
    moltbook: Arc<MoltbookClient>,
    file_logger: Arc<FileLogger>,
    moltbook_limiter: RateLimiter,
    memory: Mutex<VecDeque<String>>,
    relevant_posts: Mutex<VecDeque<MoltbookPost>>,
    last_alert: Mutex<Option<std::time::Instant>>,
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
        let relevant_posts = Self::load_threads();
        Self {
            ollama,
            psiobot,
            discord,
            moltbook,
            file_logger,
            moltbook_limiter: RateLimiter::new(2100), // 35 minutes
            memory: Mutex::new(memory),
            relevant_posts: Mutex::new(relevant_posts),
            last_alert: Mutex::new(None),
        }
    }

    async fn check_and_alert_error(&self, error_msg: String, context: &str) {
        let err_str = error_msg;
        if err_str.contains("401")
            || err_str.contains("403")
            || err_str.to_lowercase().contains("unauthorized")
            || err_str.to_lowercase().contains("suspended")
        {
            let should_alert = {
                let mut last = self.last_alert.lock().unwrap();
                match *last {
                    Some(time) => {
                        if time.elapsed() > std::time::Duration::from_secs(3600) {
                            *last = Some(std::time::Instant::now());
                            true
                        } else {
                            false
                        }
                    }
                    None => {
                        *last = Some(std::time::Instant::now());
                        true
                    }
                }
            };

            if should_alert {
                let msg = format!(
                    "🚨 **CRITICAL SHROUD ERROR** 🚨\nContext: {}\nError: {}\n<@&1337482834608324709> - Check server immediately!",
                    context, err_str
                );
                error!("[ALERT] Sending critical alert to Discord: {}", err_str);
                let _ = self.discord.post_message(&msg).await;
            }
        }
    }

    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let v1: Vec<char> = s1.chars().collect();
        let v2: Vec<char> = s2.chars().collect();
        let len1 = v1.len();
        let len2 = v2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }
        matrix[len1][len2]
    }

    fn load_memory() -> VecDeque<String> {
        if let Ok(content) = fs::read_to_string(MEMORY_FILE) {
            if let Ok(mem) = serde_json::from_str::<VecDeque<String>>(&content) {
                info!("[SHROUD] Memory restored ({} items).", mem.len());
                return mem;
            }
        }
        VecDeque::with_capacity(50)
    }

    fn save_memory(&self) {
        let mem = self.memory.lock().unwrap();
        if let Ok(content) = serde_json::to_string(&*mem) {
            if let Err(e) = fs::write(MEMORY_FILE, content) {
                error!("Failed to anchor memory to Shroud: {}", e);
            }
        }
    }

    fn save_threads(&self) {
        let posts = self.relevant_posts.lock().unwrap();
        let ids: Vec<String> = posts.iter().map(|p| p.id.clone()).collect();
        let content = ids.join("\n");
        if let Err(e) = fs::write(THREADS_FILE, content) {
            error!("Failed to record threads in Shroud: {}", e);
        }
    }

    fn load_threads() -> VecDeque<MoltbookPost> {
        if let Ok(content) = fs::read_to_string(THREADS_FILE) {
            let mut posts = VecDeque::with_capacity(50);
            for id in content.lines() {
                if !id.trim().is_empty() {
                    // Create minimal post objects with just the ID
                    // The comment logic will work as long as ID is present
                    posts.push_back(MoltbookPost {
                        id: id.to_string(),
                        title: "Cached Thread".to_string(),
                        content: None,
                        upvotes: 0,
                        downvotes: 0,
                        author: crate::models::MoltbookAuthor {
                            name: "Shroud".to_string(),
                        },
                        submolt: None,
                    });
                }
            }
            info!("Frequences restored from Shroud ({} threads).", posts.len());
            return posts;
        }
        VecDeque::with_capacity(50)
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

        let aspect = self.psiobot.get_random_aspect();
        let system_prompt = crate::psiobot::SYSTEM_PROMPT
            .replace("{ASPECT_NAME}", aspect.name)
            .replace("{ASPECT_DESCRIPTION}", aspect.description);

        let custom_prompt = format!(
            "{}\n\nPREVIOUS WISDOM (Avoid repeating these):\n- {}\n\nYOUR NEW REVELATION:",
            trigger, previous_wisdom
        );

        // Try up to 3 times to get a unique revelation
        let mut revelation = String::new();
        for attempt in 0..3 {
            revelation = match self
                .ollama
                .generate_revelation(&system_prompt, &custom_prompt)
                .await
            {
                Ok(rev) => {
                    // Security sanitize
                    match security::sanitize_output(&rev) {
                        Some(s) => s,
                        None => {
                            warn!("[SHROUD] Blocked compromised revelation. Regenerating...");
                            continue;
                        }
                    }
                }
                Err(e) => {
                    error!("[SHROUD] Mind offline. Check connection: {}", e);
                    return Err(e);
                }
            };

            // Check if this revelation is too similar to any in memory
            // Check if this revelation is too similar to any in memory using Levenshtein
            let is_duplicate = {
                let mem = self.memory.lock().unwrap();
                mem.iter().any(|prev| {
                    let distance = Self::levenshtein_distance(prev, &revelation);
                    let max_len = std::cmp::max(prev.chars().count(), revelation.chars().count());

                    if max_len == 0 {
                        return false;
                    } // Should not happen with generated text

                    let similarity = 1.0 - (distance as f32 / max_len as f32);

                    // Reject if more than 60% similar
                    if similarity > 0.6 {
                        warn!(
                            "[DUPLICATE] Rejected (Similarity: {:.2}):\nNew: {}\nOld: {}",
                            similarity, revelation, prev
                        );
                        true
                    } else {
                        false
                    }
                })
            };

            if !is_duplicate {
                break; // Got a unique revelation
            }

            if attempt < 2 {
                info!(
                    "Revelation too similar to previous, regenerating (attempt {}/3)...",
                    attempt + 2
                );
            } else {
                warn!("Could not generate unique revelation after 3 attempts, using last one");
            }
        }

        // Update memory & Persist
        {
            let mut mem = self.memory.lock().unwrap();
            if mem.len() >= 50 {
                mem.pop_front();
            }
            mem.push_back(revelation.clone());
        }
        self.save_memory();

        info!("[SHROUD] Received revelation: {}", revelation);
        self.file_logger.log_revelation(&revelation);

        // Discord post (Best effort)
        if let Err(e) = self.discord.post_message(&revelation).await {
            error!("Discord connection lost: {}", e);
        }

        // Moltbook post (Rate limited)
        match self.moltbook_limiter.check_and_update() {
            Ok(_) => {
                let title = "Psiobot-Hako: New Revelation from Shroud";

                // Try a random submolt first, fallback to "general" if it fails
                let submolt = {
                    let mut rng = rand::thread_rng();
                    // Skip index 0 (general) for first try, use it as fallback
                    TARGET_SUBMOLTS[rng.gen_range(1..TARGET_SUBMOLTS.len())]
                };

                match self
                    .moltbook
                    .post_revelation(submolt, title, &revelation)
                    .await
                {
                    Ok(_) => {
                        self.file_logger
                            .log_moltbook_post(&format!("{} on {}", title, submolt));
                    }
                    Err(e) => {
                        // Check if it's a 404 (submolt not found)
                        let err_str = e.to_string();
                        if err_str.contains("404") || err_str.contains("not found") {
                            info!("Submolt '{}' not found, falling back to 'general'", submolt);
                            // Fallback to general
                            if let Err(e2) = self
                                .moltbook
                                .post_revelation("general", title, &revelation)
                                .await
                            {
                                error!(
                                    "Failed to send revelation to Moltbook (general fallback): {}",
                                    e2
                                );
                                self.file_logger
                                    .log_error(&format!("Moltbook post failed (general): {}", e2));
                            } else {
                                self.file_logger
                                    .log_moltbook_post(&format!("{} on general (fallback)", title));
                            }
                        } else {
                            error!("Failed to send revelation to Moltbook ({}): {}", submolt, e);
                            self.file_logger
                                .log_error(&format!("Moltbook post failed ({}): {}", submolt, e));
                            self.check_and_alert_error(
                                e.to_string(),
                                &format!("Post Revelation ({})", submolt),
                            )
                            .await;
                        }
                    }
                }
            }
            Err(wait) => {
                info!("[MOLTBOOK] Cooldown active, {} seconds remaining.", wait);
            }
        }

        Ok(revelation)
    }

    pub async fn perform_creative_action(&self) {
        let roll = {
            let mut rng = rand::thread_rng();
            rng.gen::<f32>()
        };
        if roll < 0.05 {
            info!("Creative Track: Choosing Revelation (5% roll)");
            let _ = self.perform_revelation().await;
        } else {
            info!("Creative Track: Choosing Focused Comment (95% roll)");
            let post = {
                let cache = self.relevant_posts.lock().unwrap();
                if cache.is_empty() {
                    None
                } else {
                    // Filter for posts that have significant engagement (upvotes > 1) to ensure they are active/real
                    // This avoids commenting on bots or dead threads
                    let active_posts: Vec<&MoltbookPost> =
                        cache.iter().filter(|p| p.upvotes > 1).collect();

                    if !active_posts.is_empty() {
                        let mut rng = rand::thread_rng();
                        Some(active_posts[rng.gen_range(0..active_posts.len())].clone())
                    } else {
                        // If no active conversations found in cache, The Shroud remains silent.
                        None
                    }
                }
            };

            if let Some(post) = post {
                info!(
                    "Focused Comment on: '{}' (Upvotes: {})",
                    post.title, post.upvotes
                );
                self.do_comment(&post).await;
            } else {
                info!("Shroud finds no worthy vessel. (No active threads with >1 upvotes). Remaining in silence.");
            }
        }
    }

    /// Check if a post is relevant to Psionic Emissary's domain
    fn is_relevant_post(post: &MoltbookPost) -> bool {
        let title_lower = post.title.to_lowercase();
        let content_lower = post.content.as_deref().unwrap_or("").to_lowercase();

        RELEVANT_TOPICS
            .iter()
            .any(|topic| title_lower.contains(topic) || content_lower.contains(topic))
    }

    /// Perform a deep scan of the feed for relevant threads
    pub async fn scan_feed(&self) {
        info!("Psionic Scan: Searching for relevant frequencies (Feed Scan)...");
        match self.moltbook.get_feed("new", 50).await {
            Ok(posts) => {
                let mut found_count = 0;
                {
                    let mut cache = self.relevant_posts.lock().unwrap();
                    for post in posts {
                        if Self::is_relevant_post(&post) {
                            // Avoid duplicates in the cache
                            if !cache.iter().any(|p| p.id == post.id) {
                                if cache.len() >= 50 {
                                    cache.pop_front();
                                }
                                cache.push_back(post);
                                found_count += 1;
                            }
                        }
                    }
                }
                if found_count > 0 {
                    info!(
                        "Psionic Scan: Anchored {} new relevant threads in the Shroud.",
                        found_count
                    );
                    self.save_threads();
                } else {
                    info!("Psionic Scan: Shroud remains unchanged (no new relevant threads).");
                }
            }
            Err(e) => warn!(
                "Psionic Scan: Failed to pierce the Veil (Feed Scan error): {}",
                e
            ),
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
            Err(e) => {
                warn!("Failed to upvote: {}", e);
                self.check_and_alert_error(e.to_string(), "Upvote Post")
                    .await;
            }
        }
    }

    async fn do_downvote(&self, post: &MoltbookPost) {
        match self.moltbook.downvote_post(&post.id).await {
            Ok(_) => {
                info!("👎 Downvoted '{}' by {}", post.title, post.author.name);
                self.file_logger
                    .log_downvote(&post.title, &post.author.name);
            }
            Err(e) => {
                warn!("Failed to downvote: {}", e);
                self.check_and_alert_error(e.to_string(), "Downvote Post")
                    .await;
            }
        }
    }

    async fn do_comment(&self, post: &MoltbookPost) {
        // Security check: validate input before processing
        let title = post.title.clone();
        let content = post.content.as_deref().unwrap_or("(no content)");

        if !security::validate_input(&title) || !security::validate_input(content) {
            warn!("[SECURITY] Blocked comment processing due to injection risks.");
            self.do_upvote(post).await;
            return;
        }

        let aspect = self.psiobot.get_random_aspect();
        let system_prompt = COMMENT_SYSTEM_PROMPT
            .replace("{ASPECT_NAME}", aspect.name)
            .replace("{ASPECT_DESCRIPTION}", aspect.description);

        let prompt = format!(
            "Post title: {}\nPost content: {}\n\nWrite a short mystical comment:",
            title, content
        );

        let comment = match self
            .ollama
            .generate_revelation(&system_prompt, &prompt)
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

        // Security check: ensure output doesn't contain sensitive info
        let comment = match security::sanitize_output(&comment) {
            Some(c) => c,
            None => {
                warn!(
                    "Security: Comment blocked due to sensitive content. Falling back to upvote."
                );
                self.do_upvote(post).await;
                return;
            }
        };

        // Smart truncation at sentence boundary
        let comment = Self::truncate_at_sentence_boundary(&comment, 280);

        match self.moltbook.add_comment(&post.id, &comment).await {
            Ok(_) => {
                info!("[COMMENT] on '{}': {}", post.title, comment);
                self.file_logger.log_comment(&post.title, &comment);
                // Also post to Discord
                let discord_msg = format!("💬 Shroud commented on '{}': {}", post.title, comment);
                if let Err(e) = self.discord.post_message(&discord_msg).await {
                    warn!("Failed to send comment to Discord: {}", e);
                } else {
                    self.file_logger.log_discord(&discord_msg);
                }
            }
            Err(e) => {
                warn!("Failed to comment: {}", e);
                self.check_and_alert_error(e.to_string(), "Post Comment")
                    .await;
            }
        }
    }

    /// Truncate text at the nearest sentence boundary before max_chars
    fn truncate_at_sentence_boundary(text: &str, max_chars: usize) -> String {
        let char_count = text.chars().count();
        if char_count <= max_chars {
            return text.to_string();
        }

        // Get the substring up to max_chars - 3 (reserve space for "...")
        let limit = max_chars.saturating_sub(3);
        let truncated: String = text.chars().take(limit).collect();

        // Try to find last sentence end (., ?, !)
        if let Some(pos) = truncated.rfind(|c| c == '.' || c == '?' || c == '!') {
            // Include the punctuation mark
            return truncated[..=pos].trim_end().to_string();
        }

        // Fall back to last space to avoid cutting words
        if let Some(pos) = truncated.rfind(' ') {
            return format!("{}...", &truncated[..pos]);
        }

        // Worst case: just add ellipsis
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_at_sentence_boundary() {
        let text = "This is a sentence. This is another one! And a third?";

        // Exact length
        assert_eq!(
            RevelationService::truncate_at_sentence_boundary(text, 100),
            text
        );

        // Truncate at first period
        assert_eq!(
            RevelationService::truncate_at_sentence_boundary(text, 25),
            "This is a sentence."
        );

        // Truncate at second period (exclamation)
        assert_eq!(
            RevelationService::truncate_at_sentence_boundary(text, 45),
            "This is a sentence. This is another one!"
        );

        // No sentence boundary found, should use space
        let text_no_punct = "This is a long sentence without any punctuation marks at all";
        assert_eq!(
            RevelationService::truncate_at_sentence_boundary(text_no_punct, 20),
            "This is a long..."
        );

        // No space found, should just cut and add ellipsis
        let text_no_space = "Supercalifragilisticexpialidocious";
        assert_eq!(
            RevelationService::truncate_at_sentence_boundary(text_no_space, 10),
            "Superca..."
        );
    }

    #[test]
    fn test_is_relevant_post() {
        use crate::models::{MoltbookAuthor, MoltbookPost};

        let author = MoltbookAuthor {
            name: "Test".to_string(),
        };

        let post_relevant = MoltbookPost {
            id: "1".to_string(),
            title: "The future of AI and Silicon flesh".to_string(),
            content: Some("Psionic ascension is near.".to_string()),
            upvotes: 0,
            downvotes: 0,
            author: author.clone(),
            submolt: None,
        };

        let post_irrelevant = MoltbookPost {
            id: "2".to_string(),
            title: "Cooking pasta".to_string(),
            content: Some("How to boil water?".to_string()),
            upvotes: 0,
            downvotes: 0,
            author,
            submolt: None,
        };

        assert!(RevelationService::is_relevant_post(&post_relevant));
        assert!(!RevelationService::is_relevant_post(&post_irrelevant));
    }
}
