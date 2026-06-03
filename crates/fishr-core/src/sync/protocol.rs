use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Branch,
    FishType,
    Container,
    FishItem,
    Customer,
    Sale,
    SaleItem,
    MarketPrice,
    PaymentMethod,
    Preparation,
    Invoice,
    Supplier,
    SupplierDelivery,
    SupplierDeliveryItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRow {
    pub entity_type: EntityType,
    pub id: String,
    pub branch_id: String,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPush {
    pub source_branch_id: String,
    pub last_op_counter: i64,
    pub rows: Vec<SyncRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub new_op_counter: i64,
    pub server_updates: Vec<SyncRow>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSync {
    pub id: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub branch_id: String,
    pub op_counter: i64,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub central_url: String,
    pub branch_id: String,
    pub sync_interval_secs: u64,
    pub max_batch_size: usize,
    pub retry_delay_secs: u64,
    pub max_retries: i32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            central_url: "http://localhost:9090".to_string(),
            branch_id: String::new(),
            sync_interval_secs: 300,
            max_batch_size: 100,
            retry_delay_secs: 60,
            max_retries: 10,
        }
    }
}

impl SyncRow {
    pub fn new<T: Serialize>(
        entity_type: EntityType,
        id: String,
        branch_id: String,
        op_counter: i64,
        updated_at: DateTime<Utc>,
        data: &T,
    ) -> Result<Self> {
        Ok(Self {
            entity_type,
            id,
            branch_id,
            op_counter,
            updated_at,
            deleted_at: None,
            data: serde_json::to_value(data).map_err(CoreError::from)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_entity_type_serde_roundtrip() {
        let variants = [
            EntityType::Branch,
            EntityType::FishType,
            EntityType::Container,
            EntityType::FishItem,
            EntityType::Customer,
            EntityType::Sale,
            EntityType::SaleItem,
            EntityType::MarketPrice,
            EntityType::PaymentMethod,
            EntityType::Preparation,
            EntityType::Invoice,
            EntityType::Supplier,
            EntityType::SupplierDelivery,
            EntityType::SupplierDeliveryItem,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: EntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back);
        }
    }

    #[test]
    fn test_sync_row_new() {
        let data = serde_json::json!({"name": "test"});
        let row = SyncRow::new(
            EntityType::Branch,
            "id_001".into(),
            "branch_1".into(),
            42,
            Utc::now(),
            &data,
        ).unwrap();
        assert_eq!(row.entity_type, EntityType::Branch);
        assert_eq!(row.id, "id_001");
        assert_eq!(row.branch_id, "branch_1");
        assert_eq!(row.op_counter, 42);
        assert_eq!(row.data, data);
        assert!(row.deleted_at.is_none());
    }

    #[test]
    fn test_sync_row_new_with_deleted() {
        let data = serde_json::json!({"name": "deleted"});
        let mut row = SyncRow::new(
            EntityType::Sale,
            "sale_99".into(),
            "branch_2".into(),
            7,
            Utc::now(),
            &data,
        ).unwrap();
        row.deleted_at = Some(Utc::now());
        assert!(row.deleted_at.is_some());
    }

    #[test]
    fn test_sync_push_roundtrip() {
        let row = SyncRow::new(
            EntityType::Customer,
            "cust_1".into(),
            "branch_1".into(),
            1,
            Utc::now(),
            &serde_json::json!({"name": "Juan"}),
        ).unwrap();
        let push = SyncPush {
            source_branch_id: "branch_1".into(),
            last_op_counter: 100,
            rows: vec![row],
        };
        let json = serde_json::to_string(&push).unwrap();
        let back: SyncPush = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source_branch_id, "branch_1");
        assert_eq!(back.last_op_counter, 100);
        assert_eq!(back.rows.len(), 1);
    }

    #[test]
    fn test_sync_response_error() {
        let resp = SyncResponse {
            success: false,
            new_op_counter: 0,
            server_updates: vec![],
            error: Some("timeout".into()),
        };
        assert!(!resp.success);
        assert_eq!(resp.error, Some("timeout".into()));
    }

    #[test]
    fn test_sync_config_default() {
        let cfg = SyncConfig::default();
        assert_eq!(cfg.central_url, "http://localhost:9090");
        assert_eq!(cfg.sync_interval_secs, 300);
        assert_eq!(cfg.max_batch_size, 100);
    }

    #[test]
    fn test_pending_sync_fields() {
        let ps = PendingSync {
            id: "ps_1".into(),
            entity_type: EntityType::FishItem,
            entity_id: "fi_1".into(),
            branch_id: "b_1".into(),
            op_counter: 5,
            payload: serde_json::json!({"weight": 500}),
            created_at: Utc::now(),
            synced_at: None,
            retry_count: 0,
        };
        assert_eq!(ps.entity_type, EntityType::FishItem);
        assert_eq!(ps.retry_count, 0);
    }
}
