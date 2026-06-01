use axum::{
    Extension,
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait as _, EntityTrait, Set,
};
use serde::{Deserialize, Serialize};

use super::{AdminState, ApiOk};
use crate::db::entity;

#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    pub name: String,
    pub secret: String,
    pub max_use_hours: i64,
}

#[derive(Debug, Serialize)]
pub struct SecretInfo {
    pub id: i32,
    pub name: String,
    pub secret: String,
    pub created_by: String,
    pub is_active: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SecretListResponse {
    pub secrets: Vec<SecretInfo>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSecretRequest {
    pub id: i32,
}

pub async fn handle_list_secrets(
    Extension(state): Extension<AdminState>,
) -> Result<Json<SecretListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let secrets = entity::network_secrets::Entity::find()
        .all(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    let list: Vec<SecretInfo> = secrets
        .into_iter()
        .map(|s| SecretInfo {
            id: s.id,
            name: s.name,
            secret: s.secret,
            created_by: s.created_by,
            is_active: s.is_active,
            expires_at: s.expires_at.map(|t| t.to_rfc3339()),
            created_at: s.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(SecretListResponse { secrets: list }))
}

pub async fn handle_create_secret(
    Extension(state): Extension<AdminState>,
    Json(req): Json<CreateSecretRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now().fixed_offset();
    let expires_at = if req.max_use_hours < 0 {
        now + Duration::hours(876000) // ~100 years
    } else {
        now + Duration::hours(req.max_use_hours)
    };

    let secret = entity::network_secrets::ActiveModel {
        name: Set(req.name),
        secret: Set(req.secret),
        created_by: Set(state.admin_username.clone()),
        is_active: Set(true),
        expires_at: Set(Some(expires_at)),
        created_at: Set(now),
        ..Default::default()
    };

    entity::network_secrets::Entity::insert(secret)
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

pub async fn handle_delete_secret(
    Extension(state): Extension<AdminState>,
    Json(req): Json<DeleteSecretRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    entity::network_secrets::Entity::delete_by_id(req.id)
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    tracing::info!("Admin {} deleted network secret id={}", state.admin_username, req.id);
    Ok(Json(ApiOk { ok: true }))
}
