use std::collections::HashMap;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use rayon::prelude::*;
use chrono::Timelike;
use crate::api::error::ApiResult;
use crate::state::AppState;

#[derive(Serialize)]
pub struct DailyReport {
    pub date: String,
    pub total_sales: i64,
    pub total_revenue: f64,
    pub total_prep_fees: f64,
    pub average_sale: f64,
    pub by_payment_method: Vec<MethodSummary>,
    pub top_products: Vec<ProductSummary>,
    pub hourly_breakdown: Vec<HourlyData>,
}

#[derive(Serialize)]
pub struct MethodSummary {
    pub method: String,
    pub count: i64,
    pub total: f64,
}

#[derive(Serialize)]
pub struct ProductSummary {
    pub product: String,
    pub quantity: i64,
    pub revenue: f64,
}

#[derive(Serialize)]
pub struct HourlyData {
    pub hour: i32,
    pub sales: i64,
    pub revenue: f64,
}

#[derive(Serialize)]
pub struct InventoryValuation {
    pub total_value: f64,
    pub total_cost: f64,
    pub potential_margin: f64,
    pub by_container: Vec<ContainerValue>,
}

#[derive(Serialize)]
pub struct ContainerValue {
    pub container_label: String,
    pub fish_type: String,
    pub count: i32,
    pub avg_weight_g: f64,
    pub price_per_kg: f64,
    pub estimated_value: f64,
}

#[derive(Deserialize)]
pub struct SalesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_sales(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SalesQuery>,
) -> ApiResult<serde_json::Value> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, SaleRow>(
        "SELECT id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
                subtotal, preparation_fee, total, item_count, created_at, op_counter, updated_at, synced_at, deleted_at
         FROM sale WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    let sales: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "customer_name": r.customer_name,
            "payment_method": r.payment_method_name,
            "total": r.total,
            "item_count": r.item_count,
            "created_at": r.created_at,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "sales": sales, "count": sales.len() })))
}

pub async fn get_sale(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> ApiResult<serde_json::Value> {
    let sale = sqlx::query_as::<_, SaleRow>(
        "SELECT id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
                subtotal, preparation_fee, total, item_count, created_at, op_counter, updated_at, synced_at, deleted_at
         FROM sale WHERE id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db.pool)
    .await?;

    let items = sqlx::query_as::<_, SaleItemRow>(
        "SELECT id, branch_id, sale_id, fish_item_id, container_id, container_label,
                fish_type_id, fish_type_name, weight_grams, price_per_kg, preparation_id, preparation_name,
                preparation_fee, subtotal, op_counter, updated_at, synced_at, deleted_at
         FROM sale_item WHERE sale_id = ?1"
    )
    .bind(&id)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "sale": {
            "id": sale.id,
            "customer_name": sale.customer_name,
            "payment_method": sale.payment_method_name,
            "subtotal": sale.subtotal,
            "preparation_fee": sale.preparation_fee,
            "total": sale.total,
            "item_count": sale.item_count,
            "created_at": sale.created_at,
        },
        "items": items.iter().map(|i| serde_json::json!({
            "fish_type": i.fish_type_name,
            "weight_grams": i.weight_grams,
            "price_per_kg": i.price_per_kg,
            "preparation": i.preparation_name,
            "preparation_fee": i.preparation_fee,
            "subtotal": i.subtotal,
        })).collect::<Vec<_>>()
    })))
}

pub async fn daily_report(
    State(state): State<Arc<AppState>>,
) -> ApiResult<DailyReport> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let sales = sqlx::query_as::<_, SaleRow>(
        "SELECT id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
                subtotal, preparation_fee, total, item_count, created_at, op_counter, updated_at, synced_at, deleted_at
         FROM sale WHERE deleted_at IS NULL AND date(created_at) = date('now')
         ORDER BY created_at"
    )
    .fetch_all(&state.db.pool)
    .await?;

    if sales.is_empty() {
        return Ok(Json(DailyReport {
            date: today,
            total_sales: 0,
            total_revenue: 0.0,
            total_prep_fees: 0.0,
            average_sale: 0.0,
            by_payment_method: vec![],
            top_products: vec![],
            hourly_breakdown: (0..24).map(|h| HourlyData { hour: h, sales: 0, revenue: 0.0 }).collect(),
        }));
    }

    let items = sqlx::query_as::<_, SaleItemRow>(
        "SELECT id, branch_id, sale_id, fish_item_id, container_id, container_label,
                fish_type_id, fish_type_name, weight_grams, price_per_kg, preparation_id, preparation_name,
                preparation_fee, subtotal, op_counter, updated_at, synced_at, deleted_at
         FROM sale_item WHERE sale_id IN (SELECT id FROM sale WHERE date(created_at) = date('now'))"
    )
    .fetch_all(&state.db.pool)
    .await?;

    let total_revenue: f64 = sales.par_iter()
        .map(|s| s.total.parse::<f64>().unwrap_or(0.0))
        .sum();

    let total_prep: f64 = sales.par_iter()
        .map(|s| s.preparation_fee.parse::<f64>().unwrap_or(0.0))
        .sum();

    let by_method: Vec<MethodSummary> = sales.par_iter()
        .fold(HashMap::new, |mut map, s| {
            let rev = s.total.parse::<f64>().unwrap_or(0.0);
            let e = map.entry(s.payment_method_name.clone())
                .or_insert((0i64, 0.0f64));
            e.0 += 1;
            e.1 += rev;
            map
        })
        .reduce(HashMap::new, |mut a, b| {
            for (k, v) in b { let e = a.entry(k).or_insert((0, 0.0)); e.0 += v.0; e.1 += v.1; }
            a
        })
        .into_iter()
        .map(|(method, (count, total))| MethodSummary { method: method.clone(), count, total })
        .collect();

    let top_products: Vec<ProductSummary> = items.par_iter()
        .fold(HashMap::new, |mut map, i| {
            let rev = i.subtotal.parse::<f64>().unwrap_or(0.0);
            let e = map.entry(i.fish_type_name.clone())
                .or_insert((0i64, 0.0f64));
            e.0 += 1;
            e.1 += rev;
            map
        })
        .reduce(HashMap::new, |mut a, b| {
            for (k, v) in b { let e = a.entry(k).or_insert((0, 0.0)); e.0 += v.0; e.1 += v.1; }
            a
        })
        .into_iter()
        .map(|(product, (quantity, revenue))| ProductSummary { product: product.clone(), quantity, revenue })
        .collect();

    let hourly: Vec<HourlyData> = (0..24).map(|hour| {
        let count = sales.iter().filter(|s| {
            s.created_at.parse::<chrono::DateTime<chrono::Utc>>()
                .map(|dt| dt.hour() == hour as u32)
                .unwrap_or(false)
        }).count() as i64;
        let rev = sales.par_iter()
            .filter(|s| s.created_at.parse::<chrono::DateTime<chrono::Utc>>()
                .map(|dt| dt.hour() == hour as u32)
                .unwrap_or(false))
            .map(|s| s.total.parse::<f64>().unwrap_or(0.0))
            .sum();
        HourlyData { hour, sales: count, revenue: rev }
    }).collect();

    let count = sales.len() as f64;
    Ok(Json(DailyReport {
        date: today,
        total_sales: sales.len() as i64,
        total_revenue,
        total_prep_fees: total_prep,
        average_sale: if count > 0.0 { total_revenue / count } else { 0.0 },
        by_payment_method: by_method,
        top_products,
        hourly_breakdown: hourly,
    }))
}

pub async fn inventory_valuation(
    State(state): State<Arc<AppState>>,
) -> ApiResult<InventoryValuation> {
    let containers = sqlx::query_as::<_, ContainerRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count,
                location, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM container WHERE is_active = 1 AND deleted_at IS NULL AND current_count > 0"
    )
    .fetch_all(&state.db.pool)
    .await?;

    let mut values = Vec::new();
    for c in &containers {
        let avg_weight: f64 = sqlx::query_scalar::<_, f64>(
            "SELECT COALESCE(AVG(weight_grams), 0) FROM fish_item
             WHERE container_id = ?1 AND sold_at IS NULL AND deleted_at IS NULL"
        )
        .bind(&c.id)
        .fetch_one(&state.db.pool)
        .await?;

        let price: f64 = sqlx::query_scalar::<_, String>(
            "SELECT price_per_kg FROM market_price
             WHERE fish_type_id = ?1 AND effective_to IS NULL AND deleted_at IS NULL
             LIMIT 1"
        )
        .bind(&c.fish_type_id)
        .fetch_optional(&state.db.pool)
        .await?
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

        let est_value = (avg_weight / 1000.0) * price * c.current_count as f64;
        values.push((c.label.clone(), c.fish_type_name.clone(), c.current_count, avg_weight, price, est_value));
    }

    let by_container: Vec<ContainerValue> = values.iter().map(|(l, ft, cnt, avg, pr, val)| {
        ContainerValue {
            container_label: l.clone(),
            fish_type: ft.clone(),
            count: *cnt,
            avg_weight_g: *avg,
            price_per_kg: *pr,
            estimated_value: *val,
        }
    }).collect();

    let total_value: f64 = by_container.par_iter().map(|c| c.estimated_value).sum();

    Ok(Json(InventoryValuation {
        total_value,
        total_cost: 0.0,
        potential_margin: total_value,
        by_container,
    }))
}

// Row types
#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct SaleRow {
    id: String,
    branch_id: String,
    customer_id: Option<String>,
    customer_name: Option<String>,
    payment_method_id: String,
    payment_method_name: String,
    subtotal: String,
    preparation_fee: String,
    total: String,
    item_count: i32,
    created_at: String,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct SaleItemRow {
    id: String,
    branch_id: String,
    sale_id: String,
    fish_item_id: String,
    container_id: String,
    container_label: String,
    fish_type_id: String,
    fish_type_name: String,
    weight_grams: i32,
    price_per_kg: String,
    preparation_id: Option<String>,
    preparation_name: Option<String>,
    preparation_fee: String,
    subtotal: String,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct ContainerRow {
    id: String,
    branch_id: String,
    fish_type_id: String,
    fish_type_name: String,
    label: String,
    capacity: i32,
    current_count: i32,
    location: String,
    is_active: bool,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}
