use serde::Deserialize;
use tauri::command;

use super::ollama_client;

#[derive(Deserialize, serde::Serialize)]
pub struct OllamaModel {
    pub name: String,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

/// Fetch the list of locally available Ollama models.
#[command]
pub async fn list_models() -> Result<Vec<OllamaModel>, String> {
    #[cfg(target_os = "android")]
    let tags_url = "http://10.0.2.2:11434/api/tags";
    #[cfg(not(target_os = "android"))]
    let tags_url = "http://127.0.0.1:11434/api/tags";

    let response = ollama_client()
        .get(tags_url)
        .send()
        .await
        .map_err(|e| format!("Cannot reach Ollama: {e}"))?;

    let data: OllamaTagsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse model list: {e}"))?;

    Ok(data.models)
}
