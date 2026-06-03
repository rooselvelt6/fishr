use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::models::SaleItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub id: String,
    pub branch_id: String,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub payment_method_id: String,
    pub payment_method_name: String,
    pub subtotal: Decimal,
    pub preparation_fee: Decimal,
    pub total: Decimal,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleCreate {
    pub customer_id: Option<String>,
    pub payment_method_id: String,
    pub items: Vec<SaleItemCreate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItemCreate {
    pub fish_item_id: String,
    pub container_id: String,
    pub weight_grams: i32,
    pub price_per_kg: Decimal,
    pub preparation_id: Option<String>,
    pub preparation_name: Option<String>,
    pub preparation_fee: Decimal,
}

impl Sale {
    pub fn new(
        branch_id: String,
        customer_id: Option<String>,
        customer_name: Option<String>,
        payment_method_id: String,
        payment_method_name: String,
        items: &[SaleItem],
    ) -> Self {
        Self::with_id(
            ulid::Ulid::new().to_string(),
            branch_id,
            customer_id,
            customer_name,
            payment_method_id,
            payment_method_name,
            items,
        )
    }

    pub fn with_id(
        id: String,
        branch_id: String,
        customer_id: Option<String>,
        customer_name: Option<String>,
        payment_method_id: String,
        payment_method_name: String,
        items: &[SaleItem],
    ) -> Self {
        let time = Utc::now();
        let item_count = items.len() as i32;
        let subtotal: Decimal = items.iter().map(|i| i.subtotal).sum();
        let preparation_fee: Decimal = items.iter().map(|i| i.preparation_fee).sum();

        Self {
            id,
            branch_id,
            customer_id,
            customer_name,
            payment_method_id,
            payment_method_name,
            subtotal,
            preparation_fee,
            total: subtotal + preparation_fee,
            item_count,
            created_at: time,
            op_counter: time.timestamp_millis(),
            updated_at: time,
            synced_at: None,
            deleted_at: None,
        }
    }
}
