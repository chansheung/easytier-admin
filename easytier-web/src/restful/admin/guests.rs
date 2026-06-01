use axum::{
    Extension,
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait as _, ColumnTrait as _, EntityTrait as _, QueryFilter as _, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AdminState, ApiOk};
use crate::db::entity;

#[derive(Debug, Deserialize)]
pub struct CreateGuestRequest {
    pub password: String,
    pub max_use_count: i32,
    pub expiry_hours: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateGuestResponse {
    pub ok: bool,
    pub token: String,
    pub id: i32,
}

#[derive(Debug, Serialize)]
pub struct GuestInfo {
    pub id: i32,
    pub token: String,
    pub password: String,
    pub created_by: String,
    pub use_count: i32,
    pub max_use_count: i32,
    pub is_active: bool,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct GuestListResponse {
    pub guests: Vec<GuestInfo>,
}

#[derive(Debug, Deserialize)]
pub struct RevokeGuestRequest {
    pub id: i32,
}

pub async fn handle_list_guests(
    Extension(state): Extension<AdminState>,
) -> Result<Json<GuestListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let guests = entity::guest_tokens::Entity::find()
        .all(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    let guest_list: Vec<GuestInfo> = guests
        .into_iter()
        .map(|g| GuestInfo {
            id: g.id,
            token: g.token,
            password: g.password,
            created_by: g.created_by,
            use_count: g.use_count,
            max_use_count: g.max_use_count,
            is_active: g.is_active,
            expires_at: g.expires_at.to_rfc3339(),
            created_at: g.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(GuestListResponse { guests: guest_list }))
}

pub async fn handle_create_guest(
    Extension(state): Extension<AdminState>,
    Json(req): Json<CreateGuestRequest>,
) -> Result<Json<CreateGuestResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now().fixed_offset();
    let expires_at = if req.expiry_hours < 0 {
        now + Duration::hours(876000) // ~100 years
    } else {
        now + Duration::hours(req.expiry_hours)
    };
    let token = Uuid::new_v4().to_string();

    let guest = entity::guest_tokens::ActiveModel {
        token: Set(token.clone()),
        password: Set(req.password),
        created_by: Set(state.admin_username.clone()),
        max_use_count: Set(req.max_use_count),
        use_count: Set(0),
        is_active: Set(true),
        expires_at: Set(expires_at),
        created_at: Set(now),
        ..Default::default()
    };

    let result = entity::guest_tokens::Entity::insert(guest)
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    let id = result.last_insert_id;

    tracing::info!(
        "Admin {} created guest token {} (expires: {}, max_uses: {})",
        state.admin_username, token, expires_at, req.max_use_count
    );

    Ok(Json(CreateGuestResponse {
        ok: true,
        token,
        id,
    }))
}

pub async fn handle_revoke_guest(
    Extension(state): Extension<AdminState>,
    Json(req): Json<RevokeGuestRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    let guest = entity::guest_tokens::Entity::find_by_id(req.id)
        .one(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    match guest {
        Some(g) => {
            let mut active: entity::guest_tokens::ActiveModel = g.into();
            active.is_active = Set(false);
            entity::guest_tokens::Entity::update(active)
                .exec(state.db.orm_db())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
                    )
                })?;
            tracing::info!("Admin {} revoked guest token id={}", state.admin_username, req.id);
            Ok(Json(ApiOk { ok: true }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Guest token not found"})),
        )),
    }
}


pub async fn handle_delete_guest(
    Extension(state): Extension<AdminState>,
    Json(req): Json<RevokeGuestRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    entity::guest_tokens::Entity::delete_by_id(req.id)
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;
    tracing::info!("Admin {} deleted guest token id={}", state.admin_username, req.id);
    Ok(Json(ApiOk { ok: true }))
}
