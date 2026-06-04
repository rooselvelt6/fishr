use axum::extract::State;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;
use fishr_core::genetic::engine::{FishTypeStats, InventoryPlanner, PlannerConfig, PlannerResult};
use crate::api::error::ApiResult;
use crate::state::AppState;

#[derive(Serialize)]
pub struct PlannerResponse {
    pub success: bool,
    pub result: PlannerResult,
    pub note: String,
}

pub async fn plan_inventory(
    State(state): State<Arc<AppState>>,
) -> ApiResult<PlannerResponse> {
    let now = chrono::Utc::now();
    let days_back = 30;
    let since = (now - chrono::Duration::days(days_back)).format("%Y-%m-%dT%H:%M:%S").to_string();

    // Get all fish types
    let fish_types = sqlx::query_as::<_, FishTypeRow>(
        "SELECT id, name, species, category, description, op_counter, updated_at, synced_at, deleted_at
         FROM fish_type WHERE deleted_at IS NULL"
    )
    .fetch_all(&state.db.pool)
    .await?;

    // Get current market prices
    let price_rows = sqlx::query_as::<_, PriceRow>(
        "SELECT fish_type_id, price_per_kg, cost_price
         FROM market_price WHERE effective_to IS NULL AND deleted_at IS NULL"
    )
    .fetch_all(&state.db.pool)
    .await?;

    let price_map: std::collections::HashMap<String, (f64, f64)> = price_rows.iter()
        .map(|p| (p.fish_type_id.clone(), (p.price_per_kg, p.cost_price)))
        .collect();

    // Get daily sales per fish type
    let sales_data = sqlx::query_as::<_, SalesStat>(
        "SELECT fish_type_id, fish_type_name, COUNT(*) as total_sold, COALESCE(SUM(weight_grams), 0) as total_weight
         FROM sale_item WHERE updated_at >= ?1 GROUP BY fish_type_id"
    )
    .bind(&since)
    .fetch_all(&state.db.pool)
    .await?;

    let sales_map: std::collections::HashMap<String, (i64, i64)> = sales_data.iter()
        .map(|s| (s.fish_type_id.clone(), (s.total_sold, s.total_weight)))
        .collect();

    // Build stats for each fish type
    let mut stats = Vec::new();
    for ft in &fish_types {
        let (sold, weight) = sales_map.get(&ft.id).copied().unwrap_or((0, 0));
        let avg_daily = sold as f64 / days_back as f64;
        let avg_weight_kg = if sold > 0 { (weight as f64 / sold as f64) / 1000.0 } else { 0.5 };
        let (price, cost) = price_map.get(&ft.id).copied().unwrap_or((0.0, 0.0));
        let current_stock: i32 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(current_count), 0) FROM container WHERE fish_type_id = ?1 AND deleted_at IS NULL"
        )
        .bind(&ft.id)
        .fetch_one(&state.db.pool)
        .await
        .unwrap_or(0);

        stats.push(FishTypeStats {
            fish_type_id: ft.id.clone(),
            fish_type_name: ft.name.clone(),
            avg_daily_sales: avg_daily.max(0.1),
            avg_weight_kg,
            price_per_kg: price,
            cost_per_kg: cost.max(0.01),
            current_stock,
            lead_time_days: 3.0,
        });
    }

    let config = PlannerConfig::default();
    let planner = InventoryPlanner::new(stats, config);
    let result = planner.run();

    let note = format!(
        "Plan generado basado en los últimos {} días de ventas. Confianza promedio: {:.1}%",
        days_back,
        if !result.suggestions.is_empty() {
            result.suggestions.iter().map(|s| s.confidence).sum::<f64>() / result.suggestions.len() as f64 * 100.0
        } else { 0.0 }
    );

    Ok(Json(PlannerResponse {
        success: true,
        result,
        note,
    }))
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct FishTypeRow {
    id: String,
    name: String,
    species: String,
    category: String,
    description: String,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct SalesStat {
    fish_type_id: String,
    fish_type_name: String,
    total_sold: i64,
    total_weight: i64,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct PriceRow {
    fish_type_id: String,
    price_per_kg: f64,
    cost_price: f64,
}
