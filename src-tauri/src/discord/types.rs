use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Persisted Discord bot configuration.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DiscordConfig {
    /// Discord bot token (from the Discord Developer Portal).
    pub token: String,
    /// Ollama model to use when processing Discord messages.
    pub model: String,
    /// Device ID to route commands to by default.
    /// Empty string means "this device" (run headless locally).
    pub default_target_device_id: String,
    /// Direct bridge address override ("host:port", e.g. "127.0.0.1:9876").
    /// When set, Discord commands are POSTed directly to this address,
    /// bypassing device pairing. Useful for emulator or static setups.
    #[serde(default)]
    pub target_address: String,
    /// Maps Discord user ID → SHA-256 hash key.
    /// Populated when a user runs `!link <hash_key>`.
    pub user_bindings: HashMap<String, String>,
}

impl DiscordConfig {
    pub fn is_configured(&self) -> bool {
        !self.token.is_empty() && !self.model.is_empty()
    }
}
