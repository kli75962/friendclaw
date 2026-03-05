use serde::{Deserialize, Serialize};

/// Response body for `GET /ping` — lets callers verify identity and hash key.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PingResponse {
    pub device_id: String,
    pub label: String,
    /// SHA-256 hex hash key — callers must verify this matches theirs.
    pub hash_key: String,
}

/// Online status of a single peer, returned to the frontend.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerStatus {
    pub device_id: String,
    pub label: String,
    pub address: String,
    pub online: bool,
}
