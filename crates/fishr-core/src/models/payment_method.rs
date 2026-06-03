use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub id: String,
    pub branch_id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl PaymentMethod {
    pub fn new(branch_id: String, name: String, description: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            name,
            description,
            is_active: true,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
