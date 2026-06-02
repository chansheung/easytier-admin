use axum::{
    Extension,
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait as _, ColumnTrait as _, EntityTrait as _, QueryFilter as _, Set,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::{AdminState, ApiOk};
use crate::db::entity;

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub virtual_ip: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteAgentRequest {
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub virtual_ip: String,
    pub status: String,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct AgentEntry {
    pub id: i32,
    pub name: String,
    pub virtual_ip: String,
    pub description: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_sync_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct AgentListResponse {
    pub agents: Vec<AgentEntry>,
}

pub async fn handle_list_agents(
    Extension(state): Extension<AdminState>,
) -> Result<Json<AgentListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let agents = entity::agent_node::Entity::find()
        .all(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    let list: Vec<AgentEntry> = agents
        .into_iter()
        .map(|a| AgentEntry {
            id: a.id,
            name: a.name,
            virtual_ip: a.virtual_ip,
            description: a.description,
            last_sync_at: a.last_sync_at.map(|t| t.to_rfc3339()),
            last_sync_status: a.last_sync_status,
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(AgentListResponse { agents: list }))
}

pub async fn handle_create_agent(
    Extension(state): Extension<AdminState>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now().fixed_offset();
    let entry = entity::agent_node::ActiveModel {
        name: Set(req.name),
        virtual_ip: Set(req.virtual_ip),
        description: Set(req.description),
        last_sync_at: Set(None),
        last_sync_status: Set("unknown".into()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    entity::agent_node::Entity::insert(entry)
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

pub async fn handle_delete_agent(
    Extension(state): Extension<AdminState>,
    Json(req): Json<DeleteAgentRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    entity::agent_node::Entity::delete_by_id(req.id)
        .exec(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    tracing::info!("Admin {} deleted agent id={}", state.admin_username, req.id);
    Ok(Json(ApiOk { ok: true }))
}

pub async fn handle_agent_heartbeat(
    Extension(state): Extension<AdminState>,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<ApiOk>, (StatusCode, Json<serde_json::Value>)> {
    let agent: entity::agent_node::ActiveModel = entity::agent_node::Entity::find()
        .filter(entity::agent_node::Column::VirtualIp.eq(&req.virtual_ip))
        .one(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not registered"})),
        ))?
        .into();

    let mut agent = agent;
    agent.last_sync_at = Set(Some(
        req.timestamp
            .unwrap_or_else(|| Utc::now())
            .fixed_offset(),
    ));
    agent.last_sync_status = Set(req.status);
    agent.updated_at = Set(Utc::now().fixed_offset());
    agent
        .update(state.db.orm_db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("DB error: {:?}", e)})),
            )
        })?;

    Ok(Json(ApiOk { ok: true }))
}
