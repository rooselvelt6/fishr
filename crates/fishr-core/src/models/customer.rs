use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub branch_id: String,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub rif: Option<String>,
    pub address: Option<String>,
    pub points: i64,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Customer {
    pub fn new(branch_id: String, name: String, phone: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            name,
            phone,
            email: None,
            rif: None,
            address: None,
            points: 0,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
