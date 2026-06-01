use axum::{
    Extension,
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait as _, ColumnTrait as _, EntityTrait as _, QueryFilter as _, Set,
};
use serde::{Deserialize, Serialize};

use super::{AdminState, ApiOk, BlockRequest, UnblockRequest};
use crate::client_manager::session::Location;
use crate::db::entity;

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub machine_id: String,
    pub client_url: String,
    pub user_token: String,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub location: Option<Location>,
    pub blocked: bool,
    pub blocked_reason: Option<String>,
    pub last_seen: Option<String>,
    pub os: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceInfo>,
}

pub async fn handle_list_devices(
    Extension(state): Extension<AdminState>,
) -> Result<Json<DeviceListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let sessions = state.client_mgr.list_sessions().await;

    let mut devices = Vec::new();

    let blocked_devices = entity::blocked_devices::Entity::find()
        .all(state.db.orm_db())
        .await
        .unwrap_or_default();
    let blocked_ids: Vec<String> = blocked_devices.iter().map(|b| b.machine_id.clone()).collect();
    let blocked_reasons: std::collections::HashMap<String, String> = blocked_devices.iter().map(|b| (b.machine_id.clone(), b.reason.clone())).collect();










    for session in sessions {
        let hb = state.client_mgr.get_heartbeat_requests(&session.client_url).await;
        let loc = state.client_mgr.get_machine_location(&session.client_url).await;

        let (hostname, ip, os, version) = if let Some(ref h) = hb {
            (
                Some(h.hostname.clone()),
                None,
                None,
                Some(h.easytier_version.clone()),
            )
        } else {
            (None, None, None, None)
        };

        let is_blocked = blocked_ids.contains(&session.machine_id.to_string());

        devices.push(DeviceInfo {
            machine_id: session.machine_id.to_string(),
            client_url: session.client_url.to_string(),
            user_token: session.token.clone(),
            hostname,
            ip,
            location: loc,
            blocked: is_blocked,
            blocked_reason: if is_blocked { blocked_reasons.get(&session.machine_id.to_string()).cloned() } else { None },
            last_seen: hb.as_ref().map(|h| h.report_time.clone()),
            os,
            version,
        });
    }

    Ok(Json(DeviceListResponse { devices }))
}

pub async fn handle_block_device(
    Extension(state): Extension<AdminState>,
    Json(req): Json<BlockRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    let sessions = state.client_mgr.list_sessions().await;
    let device = sessions.iter().find(|s| s.machine_id.to_string() == req.machine_id);

    let (device_id, user_id) = match device {
        Some(s) => (s.client_url.to_string(), s.user_id),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Device not found"})),
            ))
        }
    };

    let existing = entity::blocked_devices::Entity::find()
        .filter(entity::blocked_devices::Column::MachineId.eq(&req.machine_id))
        .one(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    if existing.is_some() {
        return Ok(Json(ApiOk { ok: true }));
    }

    let new_block = entity::blocked_devices::ActiveModel {
        device_id: Set(device_id),
        machine_id: Set(req.machine_id.clone()),
        user_id: Set(user_id),
        blocked_by: Set(state.admin_username.clone()),
        reason: Set(req.reason.clone()),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    };

    entity::blocked_devices::Entity::insert(new_block)
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    tracing::info!("Admin {} blocked device {}: {}", state.admin_username, req.machine_id, req.reason);

    Ok(Json(ApiOk { ok: true }))
}

pub async fn handle_unblock_device(
    Extension(state): Extension<AdminState>,
    Json(req): Json<UnblockRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    entity::blocked_devices::Entity::delete_many()
        .filter(entity::blocked_devices::Column::MachineId.eq(&req.machine_id))
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    tracing::info!("Admin {} unblocked device {}", state.admin_username, req.machine_id);

    Ok(Json(ApiOk { ok: true }))
}
