use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Json, Router, extract::State, routing::{get, post}};
use tauri::AppHandle;

use crate::session::store;
use super::exec::exec_handler;
use super::types::PingResponse;

// ── Server state ─────────────────────────────────────────────────────────────

/// Minimal state passed into axum handlers — only the AppHandle for disk reads.
pub type BridgeState = Arc<AppHandle>;

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /ping — identify this device and confirm it is reachable.
/// Callers compare the returned `hash_key` to verify it is the right peer.
async fn ping_handler(State(app): State<BridgeState>) -> Json<PingResponse> {
    let cfg = store::bootstrap(&app);
    Json(PingResponse {
        device_id: cfg.device.device_id,
        label: cfg.device.label,
        hash_key: cfg.hash_key,
    })
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Start the bridge HTTP server in the background.
/// Listens on `0.0.0.0:{port}` from the saved session config.
/// Non-blocking — spawns onto the Tauri async runtime.
pub fn start_bridge_server(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let port = {
            let cfg = store::bootstrap(&app);
            cfg.bridge_port
        };

        let state: BridgeState = Arc::new(app);

        let router = Router::new()
            .route("/ping", get(ping_handler))
            .route("/exec", post(exec_handler))
            .with_state(state);

        // Try the configured port first, then fall back to the next few ports.
        let listener = {
            let mut found = None;
            for try_port in port..=port + 10 {
                let try_addr = SocketAddr::from(([0, 0, 0, 0], try_port));
                match tokio::net::TcpListener::bind(try_addr).await {
                    Ok(l) => {
                        if try_port != port {
                            eprintln!("[bridge] port {port} busy, using {try_port} instead");
                        }
                        found = Some(l);
                        break;
                    }
                    Err(_) if try_port < port + 10 => continue,
                    Err(e) => {
                        eprintln!("[bridge] failed to bind any port {port}–{}: {e}", port + 10);
                        return;
                    }
                }
            }
            found.unwrap()
        };

        eprintln!("[bridge] listening on {}", listener.local_addr().unwrap());

        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("[bridge] server error: {e}");
        }
    });
}
