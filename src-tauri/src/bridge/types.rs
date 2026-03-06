use serde::{Deserialize, Serialize};

/// Response body for `GET /ping` — returned only when the caller's key matches.
/// Does NOT include the hash key — the caller already knows it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingResponse {
    pub device_id: String,
    pub label: String,
}

/// Query parameters for `GET /ping`.
#[derive(Deserialize)]
pub struct PingQuery {
    /// The caller's hash key — must match the local key or the request is rejected.
    pub key: String,
}

/// Online status of a single peer, returned to the frontend.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerStatus {
    pub device_id: String,
    pub label: String,
    pub address: String,
    pub online: bool,
}
