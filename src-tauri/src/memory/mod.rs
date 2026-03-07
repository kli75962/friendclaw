mod fs;
pub use fs::{bootstrap_memory, build_core_prompt, execute_memory_write, memory_dir, read_core, run_memory_command};

// ----- Tauri commands exposed to the frontend -----

/// Read one of the memory files: "core.md"
#[tauri::command]
pub fn get_memory_file(app: tauri::AppHandle, filename: String) -> Result<String, String> {
    fs::read_memory_file(&app, &filename)
}

/// Overwrite one of the memory files.
#[tauri::command]
pub fn set_memory_file(app: tauri::AppHandle, filename: String, content: String) -> Result<(), String> {
    fs::write_memory_file(&app, &filename, &content)
}

/// List all saved chats (newest first).
#[tauri::command]
pub fn list_chats(app: tauri::AppHandle) -> Vec<fs::ChatMeta> {
    fs::list_chats(&app)
}

/// Load the messages array for a specific chat id.
#[tauri::command]
pub fn load_chat_messages(app: tauri::AppHandle, id: String) -> Vec<serde_json::Value> {
    fs::load_chat_messages(&app, &id)
}

/// Register a new chat entry.
#[tauri::command]
pub fn create_chat(app: tauri::AppHandle, id: String, title: String, created_at: String) -> Result<(), String> {
    fs::create_chat(&app, &id, &title, &created_at)
}

/// Persist the messages for an existing chat.
#[tauri::command]
pub fn save_chat_messages(app: tauri::AppHandle, id: String, messages: Vec<serde_json::Value>) -> Result<(), String> {
    fs::save_chat_messages(&app, &id, messages)
}

/// Delete a chat and its messages.
#[tauri::command]
pub fn delete_chat(app: tauri::AppHandle, id: String) -> Result<(), String> {
    fs::delete_chat(&app, &id)
}
