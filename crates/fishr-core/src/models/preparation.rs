use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preparation {
    pub id: String,
    pub branch_id: String,
    pub name: String,
    pub description: String,
    pub additional_cost: Decimal,
    pub cost_type: CostType,
    pub is_active: bool,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CostType {
    #[default]
    Fixed,
    Percentage,
}

impl Preparation {
    pub fn new(
        branch_id: String,
        name: String,
        description: String,
        additional_cost: Decimal,
        cost_type: CostType,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            name,
            description,
            additional_cost,
            cost_type,
            is_active: true,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
