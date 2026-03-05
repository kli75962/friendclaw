use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use super::types::DiscordConfig;

const CONFIG_FILE: &str = "discord.json";

pub fn config_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_default().join(CONFIG_FILE)
}

/// Load Discord config from disk, with env var overrides.
/// `DISCORD_TOKEN` and `DISCORD_MODEL` always take precedence over the saved file.
pub fn load(app: &AppHandle) -> DiscordConfig {
    let path = config_path(app);
    let mut cfg: DiscordConfig = std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();

    // Env vars override whatever is on disk — useful for CLI / CI launches.
    if let Ok(token) = std::env::var("DISCORD_TOKEN") {
        if !token.is_empty() { cfg.token = token; }
    }
    if let Ok(model) = std::env::var("DISCORD_MODEL") {
        if !model.is_empty() { cfg.model = model; }
    }
    if let Ok(addr) = std::env::var("DISCORD_TARGET_ADDRESS") {
        if !addr.is_empty() { cfg.target_address = addr; }
    }

    cfg
}

/// Persist Discord config (atomic write via tmp file).
pub fn save(app: &AppHandle, config: &DiscordConfig) -> Result<(), String> {
    let path = config_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

/// Bind a Discord user ID to a hash key. Creates config if it doesn't exist.
pub fn bind_user(app: &AppHandle, discord_user_id: &str, hash_key: &str) -> Result<(), String> {
    let mut cfg = load(app);
    cfg.user_bindings
        .insert(discord_user_id.to_string(), hash_key.to_string());
    save(app, &cfg)
}
