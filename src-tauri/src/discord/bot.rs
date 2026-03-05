use std::sync::Arc;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready},
    prelude::TypeMapKey,
};
use tauri::AppHandle;

use crate::bridge::exec::route_command;
use crate::ollama::types::OllamaMessage;
use crate::session::store as session_store;
use super::config::{bind_user, load as load_config};

// ── TypeMap key for sharing AppHandle with serenity ──────────────────────────

struct AppHandleKey;

impl TypeMapKey for AppHandleKey {
    type Value = Arc<AppHandle>;
}

// ── Event handler ─────────────────────────────────────────────────────────────

pub struct BotHandler;

#[async_trait]
impl EventHandler for BotHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        eprintln!("[discord] ✅ connected as {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from the bot itself.
        if msg.author.bot {
            return;
        }

        eprintln!("[discord] message from {} in {:?}: {:?}", msg.author.name, msg.channel_id, msg.content);

        // Get AppHandle from shared data.
        let app = {
            let data = ctx.data.read().await;
            match data.get::<AppHandleKey>() {
                Some(a) => a.clone(),
                None => return,
            }
        };

        let user_id = msg.author.id.to_string();
        let content = msg.content.trim().to_string();

        // ── !link <hash_key> — bind this Discord user to a device ────────────
        // Only accepts the 64-char hex key from the app (Settings → Generate hash key).
        if let Some(raw) = content.strip_prefix("!link ") {
            let raw = raw.trim();
            if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
                let _ = msg.reply(&ctx.http,
                    "❌ Invalid key. Use the 64-character hex key from Settings → Generate hash key."
                ).await;
                return;
            }
            match bind_user(&app, &user_id, raw) {
                Ok(_) => {
                    let _ = msg.reply(&ctx.http, "✅ Linked! Your messages will now be routed to your device.").await;
                }
                Err(e) => {
                    let _ = msg.reply(&ctx.http, format!("❌ Failed to link: {e}")).await;
                }
            }
            return;
        }

        // ── !status — show which device the user is linked to ─────────────────
        if content == "!status" {
            let discord_cfg = load_config(&app);
            let session_cfg = session_store::bootstrap(&app);
            let reply = if let Some(hk) = discord_cfg.user_bindings.get(&user_id) {
                if *hk == session_cfg.hash_key {
                    format!("🟢 Linked to **{}** (this device)", session_cfg.device.label)
                } else {
                    let peer = session_cfg
                        .paired_devices
                        .iter()
                        .find(|p| {
                            // We can't directly compare device hash keys here since we
                            // only store hash_key at session level not per-peer.
                            // Show the user what we have on file.
                            let _ = p;
                            false
                        });
                    let _ = peer;
                    format!("🟡 Linked (hash key on file). Use `!link passphrase` again to re-link.")
                }
            } else {
                "⚪ Not linked. Use `!link passphrase` to link this Discord account to your device.".to_string()
            };
            let _ = msg.reply(&ctx.http, reply).await;
            return;
        }

        // ── Route message as a command ─────────────────────────────────────────
        let discord_cfg = load_config(&app);
        let session_cfg = session_store::bootstrap(&app);

        let model = if discord_cfg.model.is_empty() {
            "llama3.1".to_string()
        } else {
            discord_cfg.model.clone()
        };

        // Send a "thinking" indicator for long-running commands.
        let _ = msg.channel_id.broadcast_typing(&ctx.http).await;

        // ── Route the command to the appropriate device ──────────────────────
        // Determine target device from user binding or configured default.
        let target_device_id = 'target: {
            // If user has a binding, route to that device (or its paired device).
            if discord_cfg.user_bindings.contains_key(&user_id) {
                // Linked user: honour default_target_device_id if set, else this device.
                if !discord_cfg.default_target_device_id.is_empty() {
                    break 'target discord_cfg.default_target_device_id.clone();
                }
            }
            // Fall back to configured default or this device.
            if !discord_cfg.default_target_device_id.is_empty() {
                discord_cfg.default_target_device_id.clone()
            } else {
                session_cfg.device.device_id.clone()
            }
        };

        let history: Vec<OllamaMessage> = Vec::new();
        let result = route_command(&app, &target_device_id, &content, &model, history).await;

        let reply = if result.queued {
            "📬 Your device is offline. Command queued — it will run when the device reconnects.".to_string()
        } else if result.success {
            result.response
        } else {
            format!("❌ Error: {}", result.response)
        };

        // Discord has a 2000 char limit per message.
        for chunk in reply_chunks(&reply, 1990) {
            if msg.reply(&ctx.http, chunk).await.is_err() {
                break;
            }
        }
    }
}

/// Split a reply into ≤2000 char chunks at word boundaries.
fn reply_chunks(text: &str, limit: usize) -> Vec<&str> {
    if text.len() <= limit {
        return vec![text];
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let end = (start + limit).min(text.len());
        // Walk back to a space if we'd cut mid-word.
        let end = if end < text.len() {
            text[start..end].rfind(' ').map(|i| start + i + 1).unwrap_or(end)
        } else {
            end
        };
        chunks.push(&text[start..end]);
        start = end;
    }
    chunks
}

// ── Client builder ────────────────────────────────────────────────────────────

/// Build and start the serenity Discord client.
/// Returns `Err` if the token is invalid or the connection fails.
pub async fn run_bot(app: Arc<AppHandle>, token: &str) -> Result<(), String> {
    use serenity::model::gateway::GatewayIntents;

    eprintln!("[discord] starting bot with token prefix={}…", &token[..token.len().min(10)]);

    // MESSAGE_CONTENT is a privileged intent — must be enabled in the
    // Discord Developer Portal under Bot → Privileged Gateway Intents.
    let intents = GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(BotHandler)
        .await
        .map_err(|e| format!("Failed to create Discord client: {e}"))?;

    // Share the AppHandle with event handlers via TypeMap.
    {
        let mut data = client.data.write().await;
        data.insert::<AppHandleKey>(app);
    }

    client.start().await.map_err(|e| format!("Discord client error: {e}"))
}
