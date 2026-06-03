use axum::{extract::State, Json};
use std::sync::Arc;
use crate::AppState;

pub async fn handle_sync_push(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<fishr_core::sync::SyncPush>,
) -> Json<fishr_core::sync::SyncResponse> {
    tracing::info!(
        "Sync from branch {}: {} rows (op_counter: {})",
        payload.source_branch_id,
        payload.rows.len(),
        payload.last_op_counter,
    );

    let server_updates = Vec::new();
    let mut errors = Vec::new();

    for row in &payload.rows {
        let table = match row.entity_type {
            fishr_core::sync::EntityType::Branch => "branches",
            fishr_core::sync::EntityType::FishType => "fish_types",
            fishr_core::sync::EntityType::Container => "containers",
            fishr_core::sync::EntityType::FishItem => "fish_items",
            fishr_core::sync::EntityType::Customer => "customers",
            fishr_core::sync::EntityType::Sale => "sales",
            fishr_core::sync::EntityType::SaleItem => "sale_items",
            fishr_core::sync::EntityType::MarketPrice => "market_prices",
            fishr_core::sync::EntityType::PaymentMethod => "payment_methods",
            fishr_core::sync::EntityType::Preparation => "preparations",
            fishr_core::sync::EntityType::Invoice => "invoices",
            fishr_core::sync::EntityType::Supplier => "suppliers",
            fishr_core::sync::EntityType::SupplierDelivery => "supplier_deliveries",
            fishr_core::sync::EntityType::SupplierDeliveryItem => "supplier_delivery_items",
        };

        // Upsert logic - insert or update based on op_counter
        let upsert_sql = format!(
            "INSERT INTO {} (id, branch_id, op_counter, updated_at, data)
             VALUES ($1, $2, $3, $4, $5::jsonb)
             ON CONFLICT (id) DO UPDATE SET
               op_counter = EXCLUDED.op_counter,
               updated_at = EXCLUDED.updated_at,
               data = EXCLUDED.data
             WHERE {} .op_counter < EXCLUDED.op_counter",
            table, table
        );

        match sqlx::query(&upsert_sql)
            .bind(&row.id)
            .bind(&row.branch_id)
            .bind(row.op_counter)
            .bind(row.updated_at)
            .bind(&row.data.to_string())
            .execute(&state.db.pool)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                errors.push(format!("Error upserting {} {}: {}", table, row.id, e));
            }
        }
    }

    // Log sync
    for row in &payload.rows {
        sqlx::query(
            "INSERT INTO sync_log (id, source_branch_id, entity_type, entity_id, op_counter, created_at)
             VALUES ($1, $2, $3, $4, $5, NOW())"
        )
        .bind(ulid::Ulid::new().to_string())
        .bind(&payload.source_branch_id)
        .bind(format!("{:?}", row.entity_type))
        .bind(&row.id)
        .bind(row.op_counter)
        .execute(&state.db.pool)
        .await
        .ok();
    }

    if errors.is_empty() {
        Json(fishr_core::sync::SyncResponse {
            success: true,
            new_op_counter: payload.last_op_counter,
            server_updates,
            error: None,
        })
    } else {
        Json(fishr_core::sync::SyncResponse {
            success: false,
            new_op_counter: payload.last_op_counter,
            server_updates,
            error: Some(errors.join("; ")),
        })
    }
}
