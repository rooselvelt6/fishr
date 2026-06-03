use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub branch_id: String,
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub label: String,
    pub capacity: i32,
    pub current_count: i32,
    pub location: String,
    pub is_active: bool,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Container {
    pub fn new(
        branch_id: String,
        fish_type_id: String,
        fish_type_name: String,
        label: String,
        capacity: i32,
        location: String,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            fish_type_id,
            fish_type_name,
            label,
            capacity,
            current_count: 0,
            location,
            is_active: true,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
