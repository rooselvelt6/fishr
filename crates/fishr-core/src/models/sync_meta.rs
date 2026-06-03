use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base fields for all syncable entities
pub trait SyncEntity {
    fn id(&self) -> &str;
    fn branch_id(&self) -> &str;
    fn op_counter(&self) -> i64;
    fn updated_at(&self) -> DateTime<Utc>;
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn synced_at(&self) -> Option<DateTime<Utc>>;
    fn mark_synced(&mut self, at: DateTime<Utc>);
    fn mark_deleted(&mut self);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMeta {
    pub id: String,
    pub branch_id: String,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SyncMeta {
    pub fn new(branch_id: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            op_counter: chrono::Utc::now().timestamp_millis(),
            updated_at: chrono::Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
