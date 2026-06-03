use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishItem {
    pub id: String,
    pub branch_id: String,
    pub container_id: String,
    pub container_label: String,
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub weight_grams: i32,
    pub added_at: DateTime<Utc>,
    pub sold_at: Option<DateTime<Utc>>,
    pub sold_in_sale_id: Option<String>,
    pub supplier_delivery_item_id: Option<String>,
    pub cost_price: Option<Decimal>,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishItemCreate {
    pub container_id: String,
    pub weight_grams: i32,
}

impl FishItem {
    pub fn new(
        branch_id: String,
        container_id: String,
        container_label: String,
        fish_type_id: String,
        fish_type_name: String,
        weight_grams: i32,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            container_id,
            container_label,
            fish_type_id,
            fish_type_name,
            weight_grams,
            added_at: Utc::now(),
            sold_at: None,
            sold_in_sale_id: None,
            supplier_delivery_item_id: None,
            cost_price: None,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
