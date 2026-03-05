// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Load .secrets from the src-tauri/ directory (compile-time path, always correct).
    let secrets_path = format!("{}/.secrets", env!("CARGO_MANIFEST_DIR"));
    if dotenvy::from_filename(&secrets_path).is_err() {
        // Fallback: try relative paths if running from an unusual CWD.
        if dotenvy::from_filename(".secrets").is_err() {
            let _ = dotenvy::from_filename("../.secrets");
        }
    }

    phoneclaw_lib::run()
}
