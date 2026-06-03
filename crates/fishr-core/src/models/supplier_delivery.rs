use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierDelivery {
    pub id: String,
    pub branch_id: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub delivery_date: DateTime<Utc>,
    pub notes: String,
    pub transport_plate: String,
    pub transport_driver: String,
    pub total_cost: Decimal,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SupplierDelivery {
    pub fn new(
        branch_id: String,
        supplier_id: String,
        supplier_name: String,
        transport_plate: String,
        transport_driver: String,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            supplier_id,
            supplier_name,
            delivery_date: Utc::now(),
            notes: String::new(),
            transport_plate,
            transport_driver,
            total_cost: Decimal::ZERO,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierDeliveryItem {
    pub id: String,
    pub delivery_id: String,
    pub container_id: String,
    pub container_label: String,
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub quantity: i32,
    pub weight_grams: i32,
    pub unit_cost: Decimal,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SupplierDeliveryItem {
    pub fn new(
        delivery_id: String,
        container_id: String,
        container_label: String,
        fish_type_id: String,
        fish_type_name: String,
        quantity: i32,
        weight_grams: i32,
        unit_cost: Decimal,
    ) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            delivery_id,
            container_id,
            container_label,
            fish_type_id,
            fish_type_name,
            quantity,
            weight_grams,
            unit_cost,
            op_counter: Utc::now().timestamp_millis(),
            updated_at: Utc::now(),
            synced_at: None,
            deleted_at: None,
        }
    }
}
