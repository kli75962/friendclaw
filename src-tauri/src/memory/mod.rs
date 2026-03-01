mod fs;

pub use fs::{bootstrap_memory, build_core_prompt, execute_memory_write, memory_dir, read_core, run_memory_command};

// ----- Tauri commands exposed to the frontend -----

/// Read one of the three memory files: "core.md", "notes.md", "conversations.jsonl"
#[tauri::command]
pub fn get_memory_file(app: tauri::AppHandle, filename: String) -> Result<String, String> {
    fs::read_memory_file(&app, &filename)
}

/// Overwrite one of the three memory files.
#[tauri::command]
pub fn set_memory_file(app: tauri::AppHandle, filename: String, content: String) -> Result<(), String> {
    fs::write_memory_file(&app, &filename, &content)
}
