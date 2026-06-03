use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrice {
    pub id: String,
    pub branch_id: String,
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub price_per_kg: Decimal,
    pub cost_price: Decimal,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl MarketPrice {
    pub fn new(
        branch_id: String,
        fish_type_id: String,
        fish_type_name: String,
        price_per_kg: Decimal,
        cost_price: Decimal,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            fish_type_id,
            fish_type_name,
            price_per_kg,
            cost_price,
            effective_from: Utc::now(),
            effective_to: None,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
