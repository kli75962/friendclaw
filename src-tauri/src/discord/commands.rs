use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, State};

use super::bot::run_bot;
use super::config::{load as load_config, save as save_config};
use super::types::DiscordConfig;

// ── Managed state ─────────────────────────────────────────────────────────────

/// Tracks whether the Discord bot is running.
/// The JoinHandle is stored so we can abort the task on demand.
pub struct DiscordBotState {
    pub handle: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
    pub running: Arc<AtomicBool>,
}

impl Default for DiscordBotState {
    fn default() -> Self {
        Self {
            handle: Mutex::new(None),
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Get the current Discord bot configuration (token is masked for safety).
#[tauri::command]
pub fn get_discord_config(app: AppHandle) -> DiscordConfig {
    let mut cfg = load_config(&app);
    // Mask the token before sending to frontend.
    if !cfg.token.is_empty() {
        cfg.token = "••••••••".to_string();
    }
    cfg
}

/// Save Discord bot configuration.
/// Pass the real token — it will be stored locally on disk.
#[tauri::command]
pub fn set_discord_config(
    app: AppHandle,
    token: String,
    model: String,
    default_target_device_id: String,
) -> Result<(), String> {
    let mut cfg = load_config(&app);
    // Only update token if a real value is provided (not the masked placeholder).
    if !token.starts_with("••") {
        cfg.token = token;
    }
    cfg.model = model;
    cfg.default_target_device_id = default_target_device_id;
    save_config(&app, &cfg)
}

/// Start the Discord bot using the saved token.
/// Does nothing if the bot is already running.
#[tauri::command]
pub fn start_discord_bot(
    app: AppHandle,
    bot_state: State<'_, DiscordBotState>,
) -> Result<(), String> {
    let cfg = load_config(&app);
    if !cfg.is_configured() {
        return Err("Discord bot is not configured. Set a token and model first.".to_string());
    }

    if bot_state.running.load(Ordering::Relaxed) {
        return Err("Discord bot is already running.".to_string());
    }

    let token = cfg.token.clone();
    let app_arc = Arc::new(app);
    let running = bot_state.running.clone();

    let handle = tauri::async_runtime::spawn(async move {
        running.store(true, Ordering::Relaxed);
        if let Err(e) = run_bot(app_arc, &token).await {
            eprintln!("[discord] bot stopped: {e}");
        }
        running.store(false, Ordering::Relaxed);
    });

    let mut guard = bot_state.handle.lock().map_err(|e| e.to_string())?;
    *guard = Some(handle);
    Ok(())
}

/// Stop the running Discord bot.
#[tauri::command]
pub fn stop_discord_bot(bot_state: State<'_, DiscordBotState>) -> Result<(), String> {
    let mut guard = bot_state.handle.lock().map_err(|e| e.to_string())?;
    if let Some(handle) = guard.take() {
        handle.abort();
    }
    bot_state.running.store(false, Ordering::Relaxed);
    Ok(())
}

/// Whether the Discord bot is currently running.
#[tauri::command]
pub fn discord_bot_running(bot_state: State<'_, DiscordBotState>) -> bool {
    bot_state.running.load(Ordering::Relaxed)
}
