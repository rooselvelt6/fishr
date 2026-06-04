use std::sync::Arc;
use chrono::Utc;
use crate::state::AppState;
use crate::sync::{SyncConfig, SyncError};

pub struct SyncAgent {
    state: Arc<AppState>,
    config: SyncConfig,
    client: reqwest::Client,
}

impl SyncAgent {
    pub fn new(state: Arc<AppState>, config: SyncConfig) -> Result<Self, SyncError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| SyncError::Network(format!("Error al crear cliente HTTP: {}", e)))?;

        Ok(Self { state, config, client })
    }

    pub async fn sync_once(&self) -> Result<(), SyncError> {
        let pending = sqlx::query_as::<_, PendingRow>(
            "SELECT id, entity_type, entity_id, branch_id, op_counter, payload, created_at, synced_at, retry_count
             FROM pending_sync WHERE synced_at IS NULL AND retry_count < ?1
             ORDER BY op_counter ASC LIMIT ?2"
        )
        .bind(self.config.max_retries)
        .bind(self.config.max_batch_size as i32)
        .fetch_all(&self.state.db.pool)
        .await?;

        if pending.is_empty() {
            return Ok(());
        }

        let rows: Vec<fishr_core::sync::SyncRow> = pending.iter().map(|p| {
            let entity_type = match p.entity_type.as_str() {
                "Branch" => fishr_core::sync::EntityType::Branch,
                "FishType" => fishr_core::sync::EntityType::FishType,
                "Container" => fishr_core::sync::EntityType::Container,
                "FishItem" => fishr_core::sync::EntityType::FishItem,
                "Customer" => fishr_core::sync::EntityType::Customer,
                "Sale" => fishr_core::sync::EntityType::Sale,
                "SaleItem" => fishr_core::sync::EntityType::SaleItem,
                "MarketPrice" => fishr_core::sync::EntityType::MarketPrice,
                "PaymentMethod" => fishr_core::sync::EntityType::PaymentMethod,
                "Preparation" => fishr_core::sync::EntityType::Preparation,
                "Invoice" => fishr_core::sync::EntityType::Invoice,
                "Supplier" => fishr_core::sync::EntityType::Supplier,
                "SupplierDelivery" => fishr_core::sync::EntityType::SupplierDelivery,
                "SupplierDeliveryItem" => fishr_core::sync::EntityType::SupplierDeliveryItem,
                _ => fishr_core::sync::EntityType::Branch,
            };

            fishr_core::sync::SyncRow {
                entity_type,
                id: p.entity_id.clone(),
                branch_id: p.branch_id.clone(),
                op_counter: p.op_counter,
                updated_at: Utc::now(),
                deleted_at: None,
                data: serde_json::from_str(&p.payload).unwrap_or(serde_json::Value::Null),
            }
        }).collect();

        let push = fishr_core::sync::SyncPush {
            source_branch_id: self.config.branch_id.clone(),
            last_op_counter: pending.last().map(|p| p.op_counter).unwrap_or(0),
            rows,
        };

        let url = format!("{}/api/sync/push", self.config.central_url);
        let resp = self.client
            .post(&url)
            .json(&push)
            .send()
            .await;

        match resp {
            Ok(response) => {
                if response.status().is_success() {
                    let now = Utc::now();
                    let ids: Vec<&str> = pending.iter().map(|p| p.id.as_str()).collect();

                    for id in &ids {
                        sqlx::query("UPDATE pending_sync SET synced_at=?1 WHERE id=?2")
                            .bind(now)
                            .bind(id)
                            .execute(&self.state.db.pool)
                            .await
                            .ok();
                    }

                    tracing::info!("Synced {} items successfully", pending.len());

                    if let Ok(sync_resp) = response.json::<fishr_core::sync::SyncResponse>().await {
                        if !sync_resp.server_updates.is_empty() {
                            tracing::info!("Received {} updates from server", sync_resp.server_updates.len());
                        }
                    }

                    Ok(())
                } else {
                    let status = response.status().as_u16();
                    tracing::warn!("Sync failed with status: {}", status);
                    Err(SyncError::Http(status))
                }
            }
            Err(e) => {
                tracing::warn!("Sync connection failed: {} (offline)", e);
                Err(SyncError::Network(e.to_string()))
            }
        }
    }

    pub async fn run_loop(self) {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(self.config.sync_interval_secs)
        );
        let mut backoff = self.config.retry_delay_secs;
        let max_backoff = 3600;

        loop {
            interval.tick().await;
            tracing::debug!("Running sync...");

            match self.sync_once().await {
                Ok(()) => {
                    backoff = self.config.retry_delay_secs;
                }
                Err(e) => {
                    if e.is_connection() {
                        tracing::debug!("Offline, will retry later");
                    } else {
                        tracing::error!("Sync error: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                        backoff = (backoff * 2).min(max_backoff);
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct PendingRow {
    id: String,
    entity_type: String,
    entity_id: String,
    branch_id: String,
    op_counter: i64,
    payload: String,
    created_at: String,
    synced_at: Option<String>,
    retry_count: i32,
}

pub async fn run_sync_loop(state: Arc<AppState>, config: SyncConfig) {
    let agent = match SyncAgent::new(state, config) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("Error al inicializar agente de sincronización: {}", e);
            return;
        }
    };
    agent.run_loop().await;
}

use axum::{Json, extract::State};

pub async fn api_sync_status(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pending_sync WHERE synced_at IS NULL"
    )
    .fetch_one(&state.db.pool)
    .await
    .unwrap_or(0);

    let last_sync: Option<String> = sqlx::query_scalar(
        "SELECT MAX(synced_at) FROM pending_sync WHERE synced_at IS NOT NULL"
    )
    .fetch_one(&state.db.pool)
    .await
    .ok()
    .flatten();

    Json(serde_json::json!({
        "pending_count": pending_count,
        "last_sync": last_sync,
        "branch_id": state.config.branch_id,
        "central_url": state.sync_config.central_url,
    }))
}

pub async fn api_trigger_sync(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let config = state.sync_config.clone();
    let agent = match SyncAgent::new(state.clone(), config) {
        Ok(a) => a,
        Err(e) => return Json(serde_json::json!({"error": format!("Error al inicializar: {}", e)})),
    };

    match agent.sync_once().await {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}
