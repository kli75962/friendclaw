mod memory;
mod ollama;
mod phone;
mod loadskills;
mod web_search;

use memory::{get_memory_file, set_memory_file};
use ollama::{chat_ollama, list_models};

/// App entry point — registers Tauri commands and starts the event loop.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(phone::plugin::init())
        .invoke_handler(tauri::generate_handler![
            chat_ollama,
            list_models,
            get_memory_file,
            set_memory_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

