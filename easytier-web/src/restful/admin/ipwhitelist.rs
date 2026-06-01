use axum::{
    Extension,
    http::StatusCode,
    Json,
};
use sea_orm::{
    EntityTrait, Set,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::{AdminState, ApiOk};
use crate::db::entity;

#[derive(Debug, Deserialize)]
pub struct CreateWhitelistRequest {
    pub ip: String,
    pub comment: Option<String>,
    pub hostname: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct WhitelistEntry {
    pub id: i32,
    pub ip: String,
    pub comment: Option<String>,
    pub hostname: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct WhitelistResponse {
    pub entries: Vec<WhitelistEntry>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteWhitelistRequest {
    pub id: i32,
}

pub async fn handle_list_whitelist(
    Extension(state): Extension<AdminState>,
) -> Result<Json<WhitelistResponse>, (StatusCode, Json<serde_json::Value>)> {
    let entries = entity::ip_whitelist::Entity::find()
        .all(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    let list: Vec<WhitelistEntry> = entries
        .into_iter()
        .map(|e| WhitelistEntry {
            id: e.id,
            ip: e.ip,
            comment: e.comment,
            hostname: if e.hostname.as_ref().map_or(true, |s| s.is_empty()) { None } else { e.hostname },
            created_by: e.created_by,
            created_at: e.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(WhitelistResponse { entries: list }))
}

pub async fn handle_create_whitelist(
    Extension(state): Extension<AdminState>,
    Json(req): Json<CreateWhitelistRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    let entry = entity::ip_whitelist::ActiveModel {
        ip: Set(req.ip),
        comment: Set(req.comment),
        hostname: Set(req.hostname),
        created_by: Set(state.admin_username.clone()),
        created_at: Set(Utc::now().fixed_offset()),
        ..Default::default()
    };

    entity::ip_whitelist::Entity::insert(entry)
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    Ok(Json(ApiOk { ok: true }))
}

pub async fn handle_delete_whitelist(
    Extension(state): Extension<AdminState>,
    Json(req): Json<DeleteWhitelistRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    entity::ip_whitelist::Entity::delete_by_id(req.id)
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    tracing::info!("Admin {} deleted ip whitelist id={}", state.admin_username, req.id);
    Ok(Json(ApiOk { ok: true }))
}
#[derive(Debug, Deserialize)]
pub struct UnbindWhitelistRequest {
    pub id: i32,
}

pub async fn handle_unbind_whitelist(
    Extension(state): Extension<AdminState>,
    Json(req): Json<UnbindWhitelistRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    use sea_orm::ActiveModelTrait as _;

    let entry: entity::ip_whitelist::ActiveModel = entity::ip_whitelist::Entity::find_by_id(req.id)
        .one(state.db.orm_db())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("DB error: {:?}", e)}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Entry not found"}))))?
        .into();

    let mut entry: entity::ip_whitelist::ActiveModel = entry;
    entry.hostname = Set(None);
    entry.update(state.db.orm_db()).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": format!("DB error: {:?}", e)})))
    })?;

    tracing::info!("Admin {} unbound hostname for ip whitelist id={}", state.admin_username, req.id);
    Ok(Json(ApiOk { ok: true }))
}