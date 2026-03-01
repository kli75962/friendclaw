use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// ── Memory file names ────────────────────────────────────────────────────────

pub const CORE_FILE: &str = "core.md";
pub const NOTES_FILE: &str = "notes.md";
pub const CONVERSATIONS_FILE: &str = "conversations.jsonl";
pub const ALLOWED_FILES: &[&str] = &[CORE_FILE, NOTES_FILE, CONVERSATIONS_FILE];

const DEFAULT_CORE: &str = "\
# Core Memory
- Language: always reply in **Traditional Chinese (繁體中文)**.
- Keep this file short.
- Write stable user facts here (name, recurring goals, preferences).
";

const DEFAULT_NOTES: &str = "\
# Notes
Use this file for detailed knowledge, UI navigation paths, and timestamped observations.
Format navigation paths as: \"To [action]: App > Screen > Element\"
";

// ── Filesystem helpers ───────────────────────────────────────────────────────

pub fn memory_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_default().join(".memory")
}

fn resolve_name(path: &str) -> Option<&str> {
    // Accept "/memories/core.md", "/.memory/core.md", "core.md", etc.
    let base = path
        .trim()
        .trim_start_matches('/')
        .trim_start_matches("memories/")
        .trim_start_matches(".memory/");
    if ALLOWED_FILES.contains(&base) {
        Some(base)
    } else {
        None
    }
}

/// Create `.memory/` directory and seed each file if it does not yet exist.
pub fn bootstrap_memory(app: &AppHandle) {
    let dir = memory_dir(app);
    let _ = std::fs::create_dir_all(&dir);
    ensure_file(&dir.join(CORE_FILE), DEFAULT_CORE);
    ensure_file(&dir.join(NOTES_FILE), DEFAULT_NOTES);
    ensure_file(&dir.join(CONVERSATIONS_FILE), "");
}

fn ensure_file(path: &PathBuf, default: &str) {
    if !path.exists() {
        let _ = std::fs::write(path, default);
    }
}

// ── Public readers (used by chat.rs for system-prompt injection) ─────────────

/// Read `core.md`.  Returns an empty string if the file is missing.
pub fn read_core(app: &AppHandle) -> String {
    std::fs::read_to_string(memory_dir(app).join(CORE_FILE)).unwrap_or_default()
}

/// Read `notes.md`.  Returns an empty string if the file is missing.
#[allow(dead_code)]
pub fn read_notes(app: &AppHandle) -> String {
    std::fs::read_to_string(memory_dir(app).join(NOTES_FILE)).unwrap_or_default()
}

/// Read any allowed memory file by name.
pub fn read_memory_file(app: &AppHandle, filename: &str) -> Result<String, String> {
    if !ALLOWED_FILES.contains(&filename) {
        return Err(format!("Unknown memory file: {filename}"));
    }
    std::fs::read_to_string(memory_dir(app).join(filename)).map_err(|e| e.to_string())
}

/// Overwrite an allowed memory file.
pub fn write_memory_file(app: &AppHandle, filename: &str, content: &str) -> Result<(), String> {
    if !ALLOWED_FILES.contains(&filename) {
        return Err(format!("Unknown memory file: {filename}"));
    }
    std::fs::write(memory_dir(app).join(filename), content).map_err(|e| e.to_string())
}

// ── Standalone write (called from background threads) ──────────────────────

/// Write a memory file given a pre-resolved `dir` path.
/// Used when the caller wants to fire-and-forget via `tokio::spawn`.
pub fn execute_memory_write(
    dir: PathBuf,
    _cmd: &str,
    path: Option<&str>,
    content: Option<&str>,
    mode: Option<&str>,
) -> Result<(), String> {
    let p = path.ok_or_else(|| "'path' required".to_string())?;
    let body = content.ok_or_else(|| "'content' required".to_string())?;
    let name = resolve_name(p).ok_or_else(|| format!("unknown memory file '{p}'"))?;
    let file_path = dir.join(name);

    let result = if mode == Some("append") {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .and_then(|mut f| f.write_all(body.as_bytes()))
    } else {
        std::fs::write(&file_path, body).map(|_| ())
    };

    result.map_err(|e| e.to_string())?;
    Ok(())
}

// ── LLM-facing memory tool executor ─────────────────────────────────────────

/// Handle a `memory` tool call from the LLM.
///
/// Commands:
/// - `view`   — read a file
/// - `create` — write a new file (overwrite)
/// - `update` — write (append or overwrite depending on `mode`)
/// - `search` — grep across memory files for `query`
pub fn run_memory_command(
    app: &AppHandle,
    command: &str,
    path: Option<&str>,
    content: Option<&str>,
    mode: Option<&str>,
    query: Option<&str>,
) -> String {
    let dir = memory_dir(app);

    match command {
        "view" => {
            let Some(p) = path else {
                return "error: 'path' is required for view".to_string();
            };
            let Some(name) = resolve_name(p) else {
                return format!("error: unknown memory file '{p}'");
            };
            std::fs::read_to_string(dir.join(name))
                .unwrap_or_else(|e| format!("error reading {name}: {e}"))
        }

        "create" | "update" => {
            match execute_memory_write(dir, command, path, content, mode) {
                Ok(()) => "ok: memory saved".to_string(),
                Err(e)  => format!("error: {e}"),
            }
        }

        "search" => {
            let Some(q) = query else {
                return "error: 'query' is required for search".to_string();
            };
            let terms: Vec<String> = q
                .to_lowercase()
                .split_whitespace()
                .map(String::from)
                .collect();

            // Search all files, or one specific file if path is given
            let files: Vec<PathBuf> = if let Some(p) = path {
                match resolve_name(p) {
                    Some(name) => vec![dir.join(name)],
                    None => return format!("error: unknown memory file '{p}'"),
                }
            } else {
                ALLOWED_FILES.iter().map(|f| dir.join(f)).collect()
            };

            let mut matches = Vec::new();
            for file_path in &files {
                let Ok(text) = std::fs::read_to_string(file_path) else {
                    continue;
                };
                let fname = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                for (i, line) in text.lines().enumerate() {
                    let lower = line.to_lowercase();
                    if terms.iter().any(|t| lower.contains(t.as_str())) {
                        matches.push(format!("{fname}:{}:{line}", i + 1));
                    }
                }
            }

            if matches.is_empty() {
                "no matches found".to_string()
            } else {
                matches.join("\n")
            }
        }

        other => format!("error: unknown memory command '{other}'"),
    }
}

// ── System-prompt helpers ────────────────────────────────────────────────────

/// Wrap core.md content for injection into the system prompt.
/// Called fresh before every LLM round (prepareCall pattern).
pub fn build_core_prompt(core: &str) -> String {
    let trimmed = core.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    format!("[CORE MEMORY — always apply these facts in your responses]\n{trimmed}")
}
