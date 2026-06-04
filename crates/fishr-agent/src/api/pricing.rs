use axum::extract::State;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use chrono::{Timelike, Datelike};
use fishr_core::fuzzy::sets::*;
use fishr_core::fuzzy::suggestions::*;
use crate::api::error::ApiResult;
use crate::api::inventory::{ContainerRow, MarketPriceRow};
use crate::state::AppState;

#[derive(Serialize)]
pub struct SuggestedPrice {
    pub fish_type_id: String,
    pub fish_type_name: String,
    pub base_price: f64,
    pub suggested_price: f64,
    pub factor: f64,
    pub reasons: Vec<String>,
}

pub async fn suggested_prices(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Vec<SuggestedPrice>> {
    let now = chrono::Utc::now();
    let hour = now.hour() as f64;

    let containers = sqlx::query_as::<_, ContainerRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count,
                location, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM container WHERE deleted_at IS NULL AND is_active = 1"
    )
    .fetch_all(&state.db.pool)
    .await?;

    let mut stock_by_type: std::collections::HashMap<String, (i64, i64)> = std::collections::HashMap::new();
    for c in &containers {
        let entry = stock_by_type.entry(c.fish_type_id.clone()).or_insert((0, 0));
        entry.0 += c.current_count as i64;
        entry.1 += c.capacity as i64;
    }

    let today_start = now.format("%Y-%m-%dT00:00:00").to_string();
    let today_sales: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sale WHERE created_at >= ?1"
    )
    .bind(&today_start)
    .fetch_one(&state.db.pool)
    .await
    .unwrap_or(0);
    let hourly_demand_pct = (today_sales as f64 / 24.0).min(100.0);

    let week_ago = (now - chrono::Duration::days(7)).format("%Y-%m-%dT%H:%M:%S").to_string();
    let total_items_sold: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sale_item WHERE updated_at >= ?1"
    )
    .bind(&week_ago)
    .fetch_one(&state.db.pool)
    .await
    .unwrap_or(1).max(1);

    let sales_by_type: std::collections::HashMap<String, i64> = sqlx::query_as::<_, (String, i64)>(
        "SELECT fish_type_id, COUNT(*) as cnt FROM sale_item WHERE updated_at >= ?1 GROUP BY fish_type_id"
    )
    .bind(&week_ago)
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .collect();

    let engine = build_pricing_engine();

    let mut results = Vec::new();

    let prices = sqlx::query_as::<_, MarketPriceRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, price_per_kg, cost_price,
                effective_from, effective_to, op_counter, updated_at, synced_at, deleted_at
         FROM market_price WHERE effective_to IS NULL AND deleted_at IS NULL"
    )
    .fetch_all(&state.db.pool)
    .await?;

    for price in &prices {
        let (total_count, total_cap) = stock_by_type.get(&price.fish_type_id).copied().unwrap_or((0, 0));
        let stock_pct = if total_cap > 0 {
            (total_count as f64 / total_cap as f64) * 100.0
        } else {
            50.0
        };

        let days_since = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&price.effective_from) {
            let dt_utc = dt.with_timezone(&chrono::Utc);
            (now - dt_utc).num_days() as f64
        } else {
            0.0
        };

        let fish_sales = sales_by_type.get(&price.fish_type_id).copied().unwrap_or(0);
        let popularity_pct = (fish_sales as f64 / total_items_sold as f64) * 100.0;

        let month = now.month();
        let seasonal_demand_pct = match month {
            12 | 1 | 2 => 70.0,
            3 | 4 => 50.0,
            5 | 6 | 7 => 40.0,
            8 | 9 => 55.0,
            10 | 11 => 60.0,
            _ => 50.0,
        };

        let input = FuzzyInput {
            stock_pct,
            hour,
            popularity_pct,
            customer_visits_pct: 0.0,
            hourly_demand_pct,
            days_since_price_change: days_since.min(30.0),
            seasonal_demand_pct,
        };

        let (factor, reasons) = compute_price_factor(&engine, &input);
        let base_price = price.price_per_kg;
        let suggested_price = base_price * factor;

        results.push(SuggestedPrice {
            fish_type_id: price.fish_type_id.clone(),
            fish_type_name: price.fish_type_name.clone(),
            base_price,
            suggested_price,
            factor,
            reasons,
        });
    }

    Ok(Json(results))
}
