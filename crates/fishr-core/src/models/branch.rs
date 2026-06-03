use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: String,
    pub name: String,
    pub address: String,
    pub phone: String,
    pub rif: String,
    pub is_active: bool,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Branch {
    pub fn new(name: String, address: String, phone: String, rif: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            name,
            address,
            phone,
            rif,
            is_active: true,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
