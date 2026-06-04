pub mod agent;

#[derive(Debug, Clone)]
pub enum SyncError {
    Db(String),
    Network(String),
    Http(u16),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::Db(msg) => write!(f, "sync DB error: {}", msg),
            SyncError::Network(msg) => write!(f, "sync network error: {}", msg),
            SyncError::Http(status) => write!(f, "sync HTTP error: {}", status),
        }
    }
}

impl From<sqlx::Error> for SyncError {
    fn from(e: sqlx::Error) -> Self {
        SyncError::Db(e.to_string())
    }
}

impl SyncError {
    pub fn is_connection(&self) -> bool {
        matches!(self, SyncError::Network(_))
    }
}

pub use fishr_core::sync::SyncConfig;

pub async fn push_sync<T: serde::Serialize>(state: &crate::state::AppState, entity_type: &str, data: &T) {
    let now = chrono::Utc::now();
    let payload = match serde_json::to_value(data) {
        Ok(v) => v,
        Err(_) => return,
    };
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    sqlx::query(
        "INSERT INTO pending_sync (id, entity_type, entity_id, branch_id, op_counter, payload, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )
    .bind(ulid::Ulid::new().to_string())
    .bind(entity_type)
    .bind(&id)
    .bind(&state.config.branch_id)
    .bind(now.timestamp_millis())
    .bind(payload.to_string())
    .bind(now)
    .execute(&state.db.pool)
    .await
    .ok();
}

pub async fn push_sync_batch<T: serde::Serialize>(state: &crate::state::AppState, entity_type: &str, items: &[T]) {
    if items.is_empty() {
        return;
    }
    let now = chrono::Utc::now();
    let branch_id = &state.config.branch_id;
    let ts = now.timestamp_millis();

    if let Ok(mut tx) = state.db.pool.begin().await {
        for item in items {
            let payload = match serde_json::to_value(item) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

            if sqlx::query(
                "INSERT INTO pending_sync (id, entity_type, entity_id, branch_id, op_counter, payload, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            )
            .bind(ulid::Ulid::new().to_string())
            .bind(entity_type)
            .bind(&id)
            .bind(branch_id)
            .bind(ts)
            .bind(payload.to_string())
            .bind(now)
            .execute(&mut *tx)
            .await
            .is_err() {
                return;
            }
        }
        tx.commit().await.ok();
    }
}
