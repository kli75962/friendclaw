use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

// ── Memory file names ────────────────────────────────────────────────────────

pub const CORE_FILE: &str = "core.md";
pub const CONVERSATIONS_FILE: &str = "conversations.jsonl";
pub const ALLOWED_FILES: &[&str] = &[CORE_FILE, CONVERSATIONS_FILE];

const DEFAULT_CORE: &str = "\
# Core Memory
- Keep this file short.
- Write stable user facts here (name, recurring goals, preferences).
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

/// Append one conversation summary entry to `conversations.jsonl` and keep only
/// the last 50. Designed to be called from `tokio::spawn` (takes owned `dir`).
pub fn append_conversation(dir: PathBuf, user_msg: String, reply: String) {
    const MAX_CONVS: usize = 50;

    let path = dir.join(CONVERSATIONS_FILE);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = existing
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect();

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Truncate by char count to safely handle CJK characters
    let user_trunc: String = user_msg.chars().take(300).collect();
    let reply_trunc: String = reply.chars().take(500).collect();

    let entry = serde_json::json!({
        "ts": ts,
        "user": user_trunc,
        "reply": reply_trunc,
    });
    lines.push(entry.to_string());

    // Keep only the last MAX_CONVS entries
    let start = lines.len().saturating_sub(MAX_CONVS);
    let output = lines[start..].join("\n") + "\n";
    let _ = std::fs::write(&path, output);
}

/// Read the last `limit` conversation entries formatted for system prompt injection.
pub fn read_recent_conversations(app: &AppHandle, limit: usize) -> String {
    let path = memory_dir(app).join(CONVERSATIONS_FILE);
    let content = std::fs::read_to_string(&path).unwrap_or_default();

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    let recent_start = lines.len().saturating_sub(limit);
    let recent = &lines[recent_start..];

    if recent.is_empty() {
        return String::new();
    }

    let parts: Vec<String> = recent
        .iter()
        .filter_map(|line| {
            let val = serde_json::from_str::<serde_json::Value>(line).ok()?;
            let user = val["user"].as_str().unwrap_or("");
            let reply = val["reply"].as_str().unwrap_or("");
            if user.is_empty() { return None; }
            Some(format!("User: {user}\nAssistant: {reply}"))
        })
        .collect();

    if parts.is_empty() {
        return String::new();
    }

    format!("[RECENT CONVERSATIONS — use for context if relevant]\n{}", parts.join("\n---\n"))
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
