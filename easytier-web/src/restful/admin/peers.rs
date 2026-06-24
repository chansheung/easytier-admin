use axum::{
    Extension,
    http::{HeaderMap, StatusCode},
    Json,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use super::AdminState;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportedPeer {
    pub peer_id: u32,
    pub ip: Option<String>,
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rx_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tx_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PeersResponse {
    pub peers: Vec<ReportedPeer>,
}

#[derive(Debug, Deserialize)]
pub struct ReportPeersRequest {
    #[serde(default)]
    pub reporter: Option<String>,
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

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 { return None; }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2], 16).ok()).collect()
}

pub async fn handle_report_peers(
    Extension(state): Extension<AdminState>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let secret = match &state.traffic_report_secret {
        Some(s) if !s.is_empty() => s.as_str(),
        _ => {
            tracing::warn!("Traffic report rejected: TRAFFIC_REPORT_SECRET not configured");
            return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Traffic reporting not configured"}))));
        }
    };
    let sig = headers.get("X-Report-Sig").and_then(|v| v.to_str().ok());
    let sig_bytes = match sig.and_then(from_hex) {
        Some(b) => b,
        None => return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Missing or invalid signature"})))),
    };
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "HMAC init failed"}))))?;
    mac.update(body.as_bytes());
    if mac.verify_slice(&sig_bytes).is_err() {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Invalid signature"}))));
    }

    let req: ReportPeersRequest = serde_json::from_str(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;

    if state.traffic_state.is_some() {
        super::traffic::process_report(&state, &req).await;
    }

    if let Some(store) = state.peer_store.as_ref() {
        let mut peers = store.peers.lock().unwrap();
        *peers = req.peers;
        tracing::info!("Reported {} peers from {}", peers.len(), req.reporter.as_deref().unwrap_or("unknown"));
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_valid() {
        let bytes = from_hex("48656c6c6f");
        assert_eq!(bytes, Some(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f])); // "Hello"
    }

    #[test]
    fn test_from_hex_empty() {
        assert_eq!(from_hex(""), Some(vec![]));
    }

    #[test]
    fn test_from_hex_invalid() {
        assert_eq!(from_hex("xyz"), None); // 奇数长度
        assert_eq!(from_hex("zz"), None); // 非hex字符
    }

    #[test]
    fn test_hmac_roundtrip() {
        let secret = "test-secret";
        let body = r#"{"reporter":"10.0.0.1","peers":[]}"#;

        // 签名（模拟 core 端）
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let sig_bytes = mac.finalize().into_bytes();
        let sig_hex: String = sig_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        // 验证（模拟 admin 端）
        let mut mac2 = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac2.update(body.as_bytes());
        let decoded = from_hex(&sig_hex).unwrap();
        assert!(mac2.verify_slice(&decoded).is_ok());

        // 错误的 body 应该验证失败
        let mut mac3 = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac3.update(b"wrong body");
        assert!(mac3.verify_slice(&decoded).is_err());
    }

    #[test]
    fn test_reported_peer_serde() {
        // 测试 rx_bytes/tx_bytes 为 None 时不出现在 JSON 中
        let peer = ReportedPeer {
            peer_id: 1,
            ip: Some("10.0.0.5".to_string()),
            hostname: Some("pc".to_string()),
            rx_bytes: None,
            tx_bytes: None,
        };
        let json = serde_json::to_string(&peer).unwrap();
        assert!(!json.contains("rx_bytes"));
        assert!(!json.contains("tx_bytes"));

        // 有值时出现
        let peer2 = ReportedPeer {
            peer_id: 1,
            ip: Some("10.0.0.5".to_string()),
            hostname: None,
            rx_bytes: Some(123),
            tx_bytes: Some(456),
        };
        let json2 = serde_json::to_string(&peer2).unwrap();
        assert!(json2.contains("rx_bytes"));
        assert!(json2.contains("123"));
        assert!(json2.contains("tx_bytes"));
        assert!(json2.contains("456"));
    }

    #[test]
    fn test_report_request_null_reporter() {
        // 测试 reporter: null 能正确反序列化为 None
        let json = r#"{"reporter": null, "peers": []}"#;
        let req: ReportPeersRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.reporter, None);

        // 测试 reporter 有值
        let json2 = r#"{"reporter": "10.0.0.1", "peers": []}"#;
        let req2: ReportPeersRequest = serde_json::from_str(json2).unwrap();
        assert_eq!(req2.reporter, Some("10.0.0.1".to_string()));

        // 测试 reporter 缺失（serde default）
        let json3 = r#"{"peers": []}"#;
        let req3: ReportPeersRequest = serde_json::from_str(json3).unwrap();
        assert_eq!(req3.reporter, None);
    }
}
