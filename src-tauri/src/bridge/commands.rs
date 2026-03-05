use tauri::AppHandle;

use crate::ollama::types::OllamaMessage;
use crate::session::store;
use super::exec::{route_command, ExecResponse};
use super::health::{check_all_peers, check_peer};
use super::types::PeerStatus;

/// Check if a single peer address is online and shares our hash key.
/// `address` should be "ip:port", e.g. "192.168.1.5:9876".
#[tauri::command]
pub async fn check_peer_online(app: AppHandle, address: String) -> bool {
    let hash_key = store::bootstrap(&app).hash_key;
    check_peer(&address, &hash_key).await
}

/// Check all paired peers from the session config and return their status.
#[tauri::command]
pub async fn get_all_peer_status(app: AppHandle) -> Vec<PeerStatus> {
    check_all_peers(&app).await
}

/// Route an LLM command to a specific device by its device ID.
///
/// - If `target_device_id` matches this device → run locally.
/// - If it matches a paired peer and the peer is online → forward via HTTP.
/// - If the peer is offline → queue the command and return `queued: true`.
#[tauri::command]
pub async fn send_to_device(
    app: AppHandle,
    target_device_id: String,
    message: String,
    model: String,
    history: Option<Vec<OllamaMessage>>,
) -> ExecResponse {
    let history = history.unwrap_or_default();
    route_command(&app, &target_device_id, &message, &model, history).await
}
