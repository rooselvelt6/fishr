use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub branch_id: String,
    pub sale_id: String,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub customer_rif: Option<String>,
    pub customer_address: Option<String>,
    pub control_number: String,
    pub total: Decimal,
    pub issued_at: DateTime<Utc>,
    pub op_counter: i64,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Invoice {
    pub fn new(
        branch_id: String,
        sale_id: String,
        customer_id: Option<String>,
        customer_name: Option<String>,
        customer_rif: Option<String>,
        customer_address: Option<String>,
        control_number: String,
        total: Decimal,
    ) -> Self {
        let time = Utc::now();
        Self {
            id: ulid::Ulid::new().to_string(),
            branch_id,
            sale_id,
            customer_id,
            customer_name,
            customer_rif,
            customer_address,
            control_number,
            total,
            issued_at: time,
            op_counter: time.timestamp_millis(),
            updated_at: time,
            synced_at: None,
            deleted_at: None,
        }
    }
}
