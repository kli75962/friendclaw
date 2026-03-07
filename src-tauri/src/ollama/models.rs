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
pub async fn list_models(app: tauri::AppHandle) -> Result<Vec<OllamaModel>, String> {
    let tags_url = super::types::ollama_tags_url(&app);

    let response = ollama_client()
        .get(&tags_url)
        .send()
        .await
        .map_err(|e| format!("Cannot reach Ollama: {e}"))?;

    let data: OllamaTagsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse model list: {e}"))?;

    Ok(data.models)
}
