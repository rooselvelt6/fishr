use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItem {
    pub id: String,
    pub branch_id: String,
    pub sale_id: String,
    pub fish_item_id: String,
    pub container_id: String,
    pub container_label: String,
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub weight_grams: i32,
    pub price_per_kg: Decimal,
    pub preparation_id: Option<String>,
    pub preparation_name: Option<String>,
    pub preparation_fee: Decimal,
    pub subtotal: Decimal,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SaleItem {
    pub fn new(
        branch_id: String,
        sale_id: String,
        fish_item_id: String,
        container_id: String,
        container_label: String,
        fish_type_id: String,
        fish_type_name: String,
        weight_grams: i32,
        price_per_kg: Decimal,
        preparation_id: Option<String>,
        preparation_name: Option<String>,
        preparation_fee: Decimal,
    ) -> Self {
        let weight_kg = Decimal::from_i128_with_scale(weight_grams as i128, 3);
        let subtotal = weight_kg * price_per_kg;

        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            sale_id,
            fish_item_id,
            container_id,
            container_label,
            fish_type_id,
            fish_type_name,
            weight_grams,
            price_per_kg,
            preparation_id,
            preparation_name,
            preparation_fee,
            subtotal,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
