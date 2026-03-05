use std::sync::OnceLock;
use reqwest::Client;
use tauri::AppHandle;

use crate::session::store;
use super::types::{PeerStatus, PingResponse};

/// Reuse the same HTTP client as the rest of the app (connection pool).
static HEALTH_CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> &'static Client {
    HEALTH_CLIENT.get_or_init(|| {
        Client::builder()
            // Short timeout — if a peer doesn't respond in 3 s it's considered offline.
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .expect("failed to build health-check HTTP client")
    })
}

/// Ping a single peer address (e.g. `"192.168.1.5:9876"`) and check if it is
/// online AND shares our hash key.
pub async fn check_peer(address: &str, expected_hash_key: &str) -> bool {
    let url = format!("http://{address}/ping");
    let Ok(resp) = client().get(&url).send().await else {
        return false;
    };
    let Ok(body) = resp.json::<PingResponse>().await else {
        return false;
    };
    // Verify the peer is using the same hash key (same session passphrase).
    body.hash_key == expected_hash_key
}

/// Check all paired peers in the session and return their online status.
pub async fn check_all_peers(app: &AppHandle) -> Vec<PeerStatus> {
    let cfg = store::bootstrap(app);
    let hash_key = cfg.hash_key.clone();

    // Spawn all peer checks concurrently to minimise latency.
    let mut tasks = Vec::with_capacity(cfg.paired_devices.len());
    for peer in cfg.paired_devices {
        let hk = hash_key.clone();
        tasks.push(tokio::spawn(async move {
            let online = check_peer(&peer.address, &hk).await;
            PeerStatus {
                device_id: peer.device_id,
                label: peer.label,
                address: peer.address,
                online,
            }
        }));
    }

    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        if let Ok(status) = task.await {
            results.push(status);
        }
    }
    results
}
