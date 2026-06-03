use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: String,
    pub branch_id: String,
    pub name: String,
    pub rif: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub address: Option<String>,
    pub contact_person: String,
    pub is_self: bool,
    pub is_active: bool,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Supplier {
    pub fn new(branch_id: String, name: String, phone: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            name,
            rif: None,
            phone,
            email: None,
            address: None,
            contact_person: String::new(),
            is_self: false,
            is_active: true,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }

    pub fn new_self_supplier(branch_id: String, name: String, rif: String) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            name,
            rif: Some(rif),
            phone: String::new(),
            email: None,
            address: None,
            contact_person: String::new(),
            is_self: true,
            is_active: true,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
