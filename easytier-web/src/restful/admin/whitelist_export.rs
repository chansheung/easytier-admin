use axum::{
    Extension,
    http::StatusCode,
    Json,
};
use sea_orm::EntityTrait;
use serde::Serialize;

use super::AdminState;
use crate::db::entity;

#[derive(Debug, Serialize)]
pub struct PublicWhitelistEntry {
    pub ip: String,
    pub hostname: Option<String>,
}

pub async fn handle_export_whitelist(
    Extension(state): Extension<AdminState>,
) -> Result<Json<Vec<PublicWhitelistEntry>>, (StatusCode, Json<serde_json::Value>)> {
    let rows = entity::ip_whitelist::Entity::find()
        .all(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    let entries: Vec<PublicWhitelistEntry> = rows
        .into_iter()
        .map(|r| {
            let hostname = r.hostname.filter(|s| !s.is_empty());
            PublicWhitelistEntry { ip: r.ip, hostname }
        })
        .collect();

    Ok(Json(entries))
}
