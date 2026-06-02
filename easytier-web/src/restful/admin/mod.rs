use axum::{
    Extension,
    http::{HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;

use crate::client_manager::ClientManager;
use crate::db::Db;

mod devices;
mod ipwhitelist;
mod peers;
mod whitelist_export;
mod agent;

pub const DEFAULT_ADMIN_USERNAME: &str = "admin";

#[derive(Clone)]
pub struct AdminState {
    pub client_mgr: Arc<ClientManager>,
    pub db: Db,
    pub admin_username: String,
    pub admin_password_hash: String,
    pub jwt_secret: Arc<String>,
    pub peer_store: Option<Arc<peers::PeerStore>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_devices: usize,
    pub active_sessions: usize,
    pub blocked_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ApiOk {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct BlockRequest {
    pub machine_id: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct UnblockRequest {
    pub machine_id: String,
}

fn verify_token(state: &AdminState, token: &str) -> Result<Claims, String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(state.jwt_secret.as_bytes())
        .map_err(|_| "Authentication failed".to_string())?;
    let claims: Claims = token
        .verify_with_key(&key)
        .map_err(|_| "Authentication failed".to_string())?;
    if claims.role != "admin" {
        return Err("Not authorized".to_string());
    }
    let now = Utc::now().timestamp() as usize;
    if claims.exp < now {
        return Err("Token expired".to_string());
    }
    Ok(claims)
}

async fn auth_middleware(
    Extension(state): Extension<AdminState>,
    headers: HeaderMap,
    req: axum::extract::Request,
    next: middleware::Next,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing Authorization header"})),
            )
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid Authorization format"})),
            )
        })?;

    match verify_token(&state, token) {
        Ok(_) => Ok(next.run(req).await),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": e})),
        )),
    }
}

async fn handle_login(
    Extension(state): Extension<AdminState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    if req.username != state.admin_username {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid credentials"})),
        ));
    }

    let verified = password_auth::verify_password(req.password, &state.admin_password_hash).is_ok();
    if !verified {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid credentials"})),
        ));
    }

    let claims = Claims {
        sub: req.username.clone(),
        exp: (Utc::now() + Duration::days(1)).timestamp() as usize,
        role: "admin".to_string(),
    };

    let key: Hmac<Sha256> = Hmac::new_from_slice(state.jwt_secret.as_bytes())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Authentication failed"}))))?;
    let token = claims
        .sign_with_key(&key)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Authentication failed"}))))?;

    Ok(Json(LoginResponse {
        token,
        username: req.username,
    }))
}

async fn handle_check(
    Extension(_state): Extension<AdminState>,
) -> Json<ApiOk> {
    Json(ApiOk { ok: true })
}

async fn handle_dashboard(
    Extension(state): Extension<AdminState>,
) -> Result<Json<DashboardStats>, (StatusCode, Json<serde_json::Value>)> {
    let sessions = state.client_mgr.list_sessions().await;
    let total_devices = sessions.len();
    let active_sessions = total_devices;

    use sea_orm::{EntityTrait as _, PaginatorTrait};
    let blocked = crate::db::entity::blocked_devices::Entity::find()
        .count(state.db.orm_db())
        .await
        .unwrap_or(0) as usize;

    let peer_count = state.peer_store.as_ref()
        .map(|s| s.peers.lock().unwrap().len())
        .unwrap_or(0);

    Ok(Json(DashboardStats {
        total_devices: total_devices + peer_count,
        active_sessions,
        blocked_count: blocked,
    }))
}

pub fn build_router(client_mgr: Arc<ClientManager>, db: Db) -> Router<Arc<ClientManager>> {
    let admin_username = std::env::var("ET_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let admin_password = std::env::var("ET_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let jwt_secret = std::env::var("ET_ADMIN_SECRET").unwrap_or_else(|_| "easytier-admin-secret-change-me".to_string());

    if admin_password == "admin123" || jwt_secret == "easytier-admin-secret-change-me" {
        tracing::warn!("Using default admin credentials. Set ET_ADMIN_PASSWORD and ET_ADMIN_SECRET environment variables for production use.");
    }

    let admin_password_hash = password_auth::generate_hash(&admin_password);

    let state = AdminState {
        client_mgr,
        db,
        admin_username,
        admin_password_hash,
        jwt_secret: Arc::new(jwt_secret),
        peer_store: Some(Arc::new(peers::PeerStore::default())),
    };

    let protected = Router::new()
        .route("/api/v1/admin/dashboard", get(handle_dashboard))
        .route("/api/v1/admin/check", get(handle_check))
        .route("/api/v1/admin/devices", get(devices::handle_list_devices))
        .route("/api/v1/admin/devices/block", post(devices::handle_block_device))
        .route("/api/v1/admin/devices/unblock", post(devices::handle_unblock_device))
        .route("/api/v1/admin/ipwhitelist", get(ipwhitelist::handle_list_whitelist))
        .route("/api/v1/admin/ipwhitelist/create", post(ipwhitelist::handle_create_whitelist))
        .route("/api/v1/admin/ipwhitelist/delete", post(ipwhitelist::handle_delete_whitelist))
        .route("/api/v1/admin/ipwhitelist/unbind", post(ipwhitelist::handle_unbind_whitelist))
        .route("/api/v1/admin/peers", get(peers::handle_list_peers))
        .route("/api/v1/admin/agents", get(agent::handle_list_agents))
        .route("/api/v1/admin/agents/create", post(agent::handle_create_agent))
        .route("/api/v1/admin/agents/delete", post(agent::handle_delete_agent))
        .route_layer(middleware::from_fn(auth_middleware));

    Router::new()
        .route("/api/v1/admin/login", post(handle_login))
        .route("/api/v1/admin/peers/report", post(peers::handle_report_peers))
        .route("/api/v1/public/whitelist.json", get(whitelist_export::handle_export_whitelist))
        .route("/api/v1/public/agents/heartbeat", post(agent::handle_agent_heartbeat))
        .merge(protected)
        .layer(Extension(state))
}
