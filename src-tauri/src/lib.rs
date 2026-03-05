mod memory;
mod ollama;
mod phone;
mod loadskills;
mod web_search;
mod session;
mod bridge;
mod queue;
mod discord;

use memory::{get_memory_file, set_memory_file};
use ollama::{chat_ollama, list_models};
use tauri::Manager;
use session::{add_paired_device, get_session, remove_paired_device, set_device_label, set_session_hash_key};
use bridge::{check_peer_online, get_all_peer_status, send_to_device, start_bridge_server};
use queue::{flush_all_pending, flush_queue, get_pending_queue, get_queue, queue_command};
use discord::{
    DiscordBotState,
    discord_bot_running, get_discord_config, set_discord_config,
    start_discord_bot, stop_discord_bot,
};

/// App entry point — registers Tauri commands and starts the event loop.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .secrets — desktop only (Android has no access to the host filesystem).
    #[cfg(not(target_os = "android"))]
    {
        let secrets_path = concat!(env!("CARGO_MANIFEST_DIR"), "/.secrets");
        let _ = dotenvy::from_filename(secrets_path);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(phone::plugin::init())
        .manage(DiscordBotState::default())
        .setup(|app| {
            // 1. Start the bridge HTTP server so peers can reach this device.
            start_bridge_server(app.handle().clone());

            // 2. On startup: try to deliver any messages that were queued while
            //    the target device was offline.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                flush_all_pending(&handle).await;
            });

            // 3. Auto-start Discord bot — desktop only.
            //    Android has no persistent network and can't load .secrets.
            #[cfg(not(target_os = "android"))]
            {
            let discord_cfg = discord::config::load(app.handle());
            eprintln!("[discord] config check: token_set={}, model={:?}",
                !discord_cfg.token.is_empty(), discord_cfg.model);
            if discord_cfg.is_configured() {
                let token = discord_cfg.token.clone();
                let model = discord_cfg.model.clone();
                let bot_state = app.state::<discord::DiscordBotState>();
                let running = bot_state.running.clone();
                let bot_handle = app.handle().clone();
                let handle = tauri::async_runtime::spawn(async move {
                    running.store(true, std::sync::atomic::Ordering::Relaxed);
                    eprintln!("[discord] auto-starting bot (model={model})");
                    if let Err(e) = discord::bot::run_bot(
                        std::sync::Arc::new(bot_handle), &token
                    ).await {
                        eprintln!("[discord] bot stopped: {e}");
                    }
                    running.store(false, std::sync::atomic::Ordering::Relaxed);
                });
                *bot_state.handle.lock().unwrap() = Some(handle);
            }
            } // cfg(not(target_os = "android"))

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            chat_ollama,
            list_models,
            get_memory_file,
            set_memory_file,
            // session / pairing
            get_session,
            set_device_label,
            set_session_hash_key,
            add_paired_device,
            remove_paired_device,
            // bridge / health
            check_peer_online,
            get_all_peer_status,
            send_to_device,
            // queue
            get_queue,
            get_pending_queue,
            queue_command,
            flush_queue,
            // discord bot
            get_discord_config,
            set_discord_config,
            start_discord_bot,
            stop_discord_bot,
            discord_bot_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

