use serde::Deserialize;
use std::sync::OnceLock;
use tauri::command;

#[derive(Deserialize, serde::Serialize)]
pub struct OllamaModel {
    pub name: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

static OLLAMA_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn ollama_client() -> &'static reqwest::Client {
    OLLAMA_CLIENT.get_or_init(reqwest::Client::new)
}

/// Fetch the list of locally available Ollama models.
#[command]
pub async fn list_models() -> Result<Vec<OllamaModel>, String> {
    let response = ollama_client()
        .get("http://10.0.2.2:11434/api/tags")
        .send()
        .await
        .map_err(|e| format!("Cannot reach Ollama: {e}"))?;

    let data: OllamaTagsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse model list: {e}"))?;

    Ok(data.models)
}
