use axum::{Extension, Json, extract::Query, http::StatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::db::entity::{ip_whitelist, traffic_quota, traffic_usage};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait};
use sea_orm::ActiveValue::Set;
use chrono::Local;

use super::AdminState;
use super::peers::ReportPeersRequest;

pub struct TrafficState {
    snapshots: Mutex<HashMap<(String, String), (u64, u64)>>,
    started_at: Instant,
}

impl TrafficState {
    pub fn new() -> Self {
        Self {
            snapshots: Mutex::new(HashMap::new()),
            started_at: Instant::now(),
        }
    }
}

fn current_period_key(period_type: &str) -> String {
    let now = Local::now();
    match period_type {
        "hour" => now.format("%Y-%m-%dT%H").to_string(),
        "day" => now.format("%Y-%m-%d").to_string(),
        "week" => now.format("%G-W%V").to_string(),
        "month" => now.format("%Y-%m").to_string(),
        _ => now.format("%Y-%m-%d").to_string(),
    }
}

pub async fn process_report(state: &AdminState, req: &ReportPeersRequest) {
    let reporter = match &req.reporter {
        Some(r) if !r.is_empty() => r.as_str(),
        _ => return,
    };

    let traffic_state = match &state.traffic_state {
        Some(ts) => ts,
        None => return,
    };

    let in_cooldown = traffic_state.started_at.elapsed() < std::time::Duration::from_secs(90);

    // Compute deltas while holding the snapshot lock, then release it before
    // doing any async DB work so the future stays `Send`.
    let deltas: Vec<(String, u64)> = {
        let mut snapshots = traffic_state.snapshots.lock().unwrap();
        let mut out = Vec::new();
        for peer in &req.peers {
            let Some(ip) = &peer.ip else { continue };
            let rx = peer.rx_bytes.unwrap_or(0);
            let tx = peer.tx_bytes.unwrap_or(0);
            if rx == 0 && tx == 0 { continue; }

            let key = (reporter.to_string(), ip.clone());
            let (delta_rx, delta_tx) = match snapshots.get(&key) {
                Some(&(prx, ptx)) => (rx.saturating_sub(prx), tx.saturating_sub(ptx)),
                None => (0u64, 0u64),
            };
            snapshots.insert(key, (rx, tx));

            if in_cooldown { continue; }
            let delta = delta_rx + delta_tx;
            if delta == 0 { continue; }
            out.push((ip.clone(), delta));
        }

        // Clean up stale entries for this reporter that are no longer being reported.
        // Without this, the snapshots map grows unbounded as peers come and go.
        let current_ips: std::collections::HashSet<&str> = req.peers.iter()
            .filter_map(|p| p.ip.as_deref())
            .collect();
        snapshots.retain(|(r, ip), _| r.as_str() != reporter || current_ips.contains(ip.as_str()));

        out
    }; // MutexGuard dropped here

    let db_conn = state.db.orm_db();
    for (ip, delta) in deltas {
        let quotas = traffic_quota::Entity::find()
            .filter(traffic_quota::Column::Ip.eq(&ip))
            .filter(traffic_quota::Column::Enabled.eq(true))
            .all(db_conn).await;

        if let Ok(quotas) = quotas {
            for quota in quotas {
                let pk = current_period_key(&quota.period_type);

                use sea_orm::sea_query::Expr;

                // Atomic UPSERT: first try to increment the existing row. If no row
                // matched (rows_affected == 0), insert a new one. This avoids the
                // find-then-update-or-insert race when multiple reports arrive concurrently.
                let delta_i = delta as i64;
                let result = traffic_usage::Entity::update_many()
                    .col_expr(traffic_usage::Column::Bytes, Expr::col(traffic_usage::Column::Bytes).add(delta_i))
                    .filter(traffic_usage::Column::Ip.eq(&ip))
                    .filter(traffic_usage::Column::PeriodType.eq(&quota.period_type))
                    .filter(traffic_usage::Column::PeriodKey.eq(&pk))
                    .exec(db_conn).await;

                let rows = result.map(|r| r.rows_affected).unwrap_or(0);
                if rows == 0 {
                    let new_row = traffic_usage::ActiveModel {
                        ip: Set(ip.clone()),
                        period_type: Set(quota.period_type.clone()),
                        period_key: Set(pk.clone()),
                        bytes: Set(delta_i),
                        updated_at: Set(Local::now().fixed_offset()),
                        ..Default::default()
                    };
                    if let Err(e) = new_row.insert(db_conn).await {
                        tracing::error!("Failed to insert traffic_usage for {}: {}", ip, e);
                    }
                }

                // Re-read current usage after the upsert to decide whether the
                // quota has been exceeded.
                let current = traffic_usage::Entity::find()
                    .filter(traffic_usage::Column::Ip.eq(&ip))
                    .filter(traffic_usage::Column::PeriodType.eq(&quota.period_type))
                    .filter(traffic_usage::Column::PeriodKey.eq(&pk))
                    .one(db_conn).await
                    .ok()
                    .flatten()
                    .map(|r| r.bytes)
                    .unwrap_or(0);

                if current >= quota.limit_bytes {
                    match ip_whitelist::Entity::delete_many()
                        .filter(ip_whitelist::Column::Ip.eq(&ip))
                        .exec(db_conn).await
                    {
                        Ok(r) => tracing::warn!("Blocked IP {} (quota exceeded: {} >= {}, period={}), deleted {} rows", ip, current, quota.limit_bytes, quota.period_type, r.rows_affected),
                        Err(e) => tracing::error!("FAILED to block IP {} after quota exceeded: {}", ip, e),
                    }
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub struct SetQuotaRequest {
    pub ip: String,
    pub period_type: String,
    pub limit_bytes: i64,
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct DeleteQuotaRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct UsageQuery {
    pub ip: Option<String>,
}

pub async fn handle_list_quotas(
    Extension(state): Extension<AdminState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let db_conn = state.db.orm_db();
    let quotas = traffic_quota::Entity::find().all(db_conn).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
    Ok(Json(serde_json::json!({"quotas": quotas})))
}

pub async fn handle_set_quota(
    Extension(state): Extension<AdminState>,
    Json(req): Json<SetQuotaRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match req.period_type.as_str() {
        "hour" | "day" | "week" | "month" => {},
        _ => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "period_type must be one of: hour, day, week, month"})))),
    }
    let db_conn = state.db.orm_db();
    let existing = traffic_quota::Entity::find()
        .filter(traffic_quota::Column::Ip.eq(&req.ip))
        .filter(traffic_quota::Column::PeriodType.eq(&req.period_type))
        .one(db_conn).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let enabled = req.enabled.unwrap_or(true);
    match existing {
        Some(row) => {
            let mut active: traffic_quota::ActiveModel = row.into();
            active.limit_bytes = Set(req.limit_bytes);
            active.enabled = Set(enabled);
            active.updated_at = Set(Local::now().fixed_offset());
            let _ = active.update(db_conn).await;
        }
        None => {
            let new_row = traffic_quota::ActiveModel {
                ip: Set(req.ip.clone()),
                period_type: Set(req.period_type.clone()),
                limit_bytes: Set(req.limit_bytes),
                enabled: Set(enabled),
                created_at: Set(Local::now().fixed_offset()),
                updated_at: Set(Local::now().fixed_offset()),
                ..Default::default()
            };
            let _ = new_row.insert(db_conn).await;
        }
    }
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn handle_delete_quota(
    Extension(state): Extension<AdminState>,
    Json(req): Json<DeleteQuotaRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let db_conn = state.db.orm_db();
    let _ = traffic_quota::Entity::delete_by_id(req.id).exec(db_conn).await;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn handle_list_usage(
    Extension(state): Extension<AdminState>,
    Query(q): Query<UsageQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let db_conn = state.db.orm_db();
    let query = traffic_usage::Entity::find();
    let usage = match q.ip {
        Some(ip) => query.filter(traffic_usage::Column::Ip.eq(&ip)).all(db_conn).await,
        None => query.all(db_conn).await,
    }.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;
    Ok(Json(serde_json::json!({"usage": usage})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_period_key_formats() {
        let now = chrono::Local::now();
        let hour = current_period_key("hour");
        assert!(hour.starts_with(&now.format("%Y-%m-%dT%H").to_string()[..10]));
        assert!(hour.len() == 13); // YYYY-MM-DDTHH

        let day = current_period_key("day");
        assert_eq!(day.len(), 10); // YYYY-MM-DD

        let week = current_period_key("week");
        assert!(week.contains("-W")); // YYYY-WNN

        let month = current_period_key("month");
        assert_eq!(month.len(), 7); // YYYY-MM

        // fallback
        let unknown = current_period_key("year");
        assert_eq!(unknown.len(), 10); // falls back to day format
    }

    #[test]
    fn test_delta_calculation() {
        // 模拟增量计算逻辑：max(0, cur - prev) 宁漏不溢
        let prev_rx: u64 = 1000;
        let prev_tx: u64 = 500;

        // 正常增量
        let cur_rx: u64 = 1500;
        let cur_tx: u64 = 800;
        let delta_rx = cur_rx.saturating_sub(prev_rx);
        let delta_tx = cur_tx.saturating_sub(prev_tx);
        assert_eq!(delta_rx, 500);
        assert_eq!(delta_tx, 300);
        assert_eq!(delta_rx + delta_tx, 800);

        // conn 重建导致计数器回退：cur < prev → Δ=0（宁漏不溢）
        let new_rx: u64 = 200; // conn 重建后从更小的值开始
        let new_tx: u64 = 100;
        let d_rx = new_rx.saturating_sub(prev_rx); // 200 - 1000 = 0 (saturating)
        let d_tx = new_tx.saturating_sub(prev_tx); // 100 - 500 = 0
        assert_eq!(d_rx, 0);
        assert_eq!(d_tx, 0);
    }

    #[test]
    fn test_traffic_state_cooldown() {
        let state = TrafficState::new();
        // 刚创建应该处于冷却期（90s内）
        assert!(state.started_at.elapsed() < std::time::Duration::from_secs(90));
    }
}
