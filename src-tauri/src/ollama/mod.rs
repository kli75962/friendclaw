pub mod chat;
pub mod headless;
pub mod models;
pub mod types;

// Re-export the Tauri commands so lib.rs can register them directly
pub use chat::{cancel_chat, chat_ollama};
pub use models::list_models;

// ── Shared HTTP client for all Ollama requests ────────────────────────────────
// One connection pool reused across chat, headless, and model list requests.
use std::sync::OnceLock;

static OLLAMA_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn ollama_client() -> &'static reqwest::Client {
    OLLAMA_CLIENT.get_or_init(reqwest::Client::new)
}
