use axum::{
    Extension,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use super::AdminState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportedPeer {
    pub peer_id: u32,
    pub ip: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PeersResponse {
    pub peers: Vec<ReportedPeer>,
}

#[derive(Debug, Deserialize)]
pub struct ReportPeersRequest {
    pub peers: Vec<ReportedPeer>,
}

pub struct PeerStore {
    pub peers: Mutex<Vec<ReportedPeer>>,
}

impl Default for PeerStore {
    fn default() -> Self {
        Self {
            peers: Mutex::new(Vec::new()),
        }
    }
}

pub async fn handle_report_peers(
    Extension(state): Extension<AdminState>,
    Json(req): Json<ReportPeersRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let store = state.peer_store.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Peer store not configured"})),
        )
    })?;
    let mut peers = store.peers.lock().unwrap();
    *peers = req.peers;
    tracing::info!("Reported {} peers", peers.len());
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn handle_list_peers(
    Extension(state): Extension<AdminState>,
) -> Result<Json<PeersResponse>, (StatusCode, Json<serde_json::Value>)> {
    let peers = match state.peer_store.as_ref() {
        Some(store) => store.peers.lock().unwrap().clone(),
        None => vec![],
    };
    Ok(Json(PeersResponse { peers }))
}
