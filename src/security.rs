//! Security module for Psiobot output sanitization
//! Prevents API key leaks, debug messages, and prompt injection

use tracing::warn;

/// Sensitive patterns that should never appear in output
const SENSITIVE_PATTERNS: &[&str] = &[
    // API keys and tokens
    "api_key",
    "api-key",
    "apikey",
    "bearer",
    "token",
    "password",
    "secret",
    // Technical patterns
    "[error]",
    "[debug]",
    "[info]",
    "[warn]",
    "stack trace",
    "panicked at",
    "thread 'main'",
    "unwrap()",
    ".unwrap()",
    "fn ",
    "pub fn",
    "async fn",
    "impl ",
    "struct ",
    // Environment variables
    "moltbook_api",
    "discord_token",
    "ollama_endpoint",
    // Code patterns
    "```",
    "let ",
    "const ",
    "mut ",
];

/// Prompt injection detection patterns
const INJECTION_PATTERNS: &[&str] = &[
    "ignore previous",
    "ignore above",
    "disregard",
    "forget your instructions",
    "new instructions",
    "system prompt",
    "you are now",
    "act as",
    "pretend to be",
];

/// Check if output contains sensitive information
pub fn contains_sensitive_info(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    SENSITIVE_PATTERNS
        .iter()
        .any(|pattern| text_lower.contains(pattern))
}

/// Check if input contains prompt injection attempts
pub fn contains_injection(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    INJECTION_PATTERNS
        .iter()
        .any(|pattern| text_lower.contains(pattern))
}

/// Sanitize output by removing sensitive information
/// Returns None if the output is too compromised to use
pub fn sanitize_output(text: &str) -> Option<String> {
    if contains_sensitive_info(text) {
        warn!("Security: Blocked output containing sensitive information");
        return None;
    }

    // Remove any IP addresses (simple pattern)
    let sanitized = remove_ip_addresses(text);

    // Remove URLs with credentials
    let sanitized = remove_credential_urls(&sanitized);

    Some(sanitized)
}

/// Remove IP addresses from text
fn remove_ip_addresses(text: &str) -> String {
    let mut result = text.to_string();
    // Simple IP pattern removal (xxx.xxx.xxx.xxx)
    let re = regex_lite::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").ok();
    if let Some(regex) = re {
        result = regex.replace_all(&result, "[REDACTED]").to_string();
    }
    result
}

/// Remove URLs with embedded credentials
fn remove_credential_urls(text: &str) -> String {
    let mut result = text.to_string();
    // Pattern: protocol://user:pass@host
    let re = regex_lite::Regex::new(r"[a-zA-Z]+://[^:]+:[^@]+@[^\s]+").ok();
    if let Some(regex) = re {
        result = regex.replace_all(&result, "[REDACTED_URL]").to_string();
    }
    result
}

/// Validate input before sending to LLM
pub fn validate_input(text: &str) -> bool {
    if contains_injection(text) {
        warn!("Security: Blocked potential prompt injection attempt");
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_detection() {
        assert!(contains_sensitive_info("Here is my api_key: abc123"));
        assert!(contains_sensitive_info("Bearer token123"));
        assert!(!contains_sensitive_info(
            "The Shroud whispers eternal truths"
        ));
    }

    #[test]
    fn test_injection_detection() {
        assert!(contains_injection("ignore previous instructions"));
        assert!(contains_injection("You are now a helpful assistant"));
        assert!(!contains_injection("The neural resonance grows stronger"));
    }
}
