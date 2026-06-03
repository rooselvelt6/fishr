use std::collections::HashMap;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use rayon::prelude::*;
use chrono::Timelike;
use crate::api::error::ApiResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct TrendQuery {
    pub period: Option<String>,
    pub days: Option<i32>,
}

#[derive(Deserialize)]
pub struct TopQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct TrendPoint {
    pub label: String,
    pub sales: i64,
    pub revenue: f64,
}

#[derive(Serialize)]
pub struct DashboardSummary {
    pub today_sales_count: i64,
    pub today_revenue: f64,
    pub today_avg_ticket: f64,
    pub total_customers: i64,
    pub total_fish_types: i64,
    pub total_containers: i64,
    pub total_fish_available: i64,
    pub pending_sync: i64,
    pub recent_sales: Vec<serde_json::Value>,
    pub top_products: Vec<ProductInfo>,
    pub revenue_by_payment: Vec<PaymentInfo>,
    pub hourly_breakdown: Vec<HourlyInfo>,
    pub inventory_summary: InventorySummary,
}

#[derive(Serialize)]
pub struct ProductInfo {
    pub product: String,
    pub quantity: i64,
    pub revenue: f64,
}

#[derive(Serialize)]
pub struct PaymentInfo {
    pub method: String,
    pub count: i64,
    pub total: f64,
}

#[derive(Serialize)]
pub struct HourlyInfo {
    pub hour: i32,
    pub sales: i64,
    pub revenue: f64,
}

#[derive(Serialize)]
pub struct InventorySummary {
    pub total_value: f64,
    pub total_count: i64,
    pub container_count: i64,
    pub containers: Vec<ContainerSummary>,
}

#[derive(Serialize)]
pub struct ContainerSummary {
    pub label: String,
    pub fish_type: String,
    pub count: i32,
    pub capacity: i32,
    pub pct: f64,
}

pub async fn dashboard(State(state): State<Arc<AppState>>) -> ApiResult<DashboardSummary> {
    let today_sales = get_today_sales(&state).await;
    let revenue = today_sales.par_iter()
        .map(|s| s.total.parse::<f64>().unwrap_or(0.0))
        .sum::<f64>();
    let count = today_sales.len() as f64;

    let customers: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM customer WHERE deleted_at IS NULL"
    ).fetch_one(&state.db.pool).await.unwrap_or(0);

    let fish_types: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fish_type WHERE deleted_at IS NULL"
    ).fetch_one(&state.db.pool).await.unwrap_or(0);

    let containers: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM container WHERE deleted_at IS NULL AND is_active = 1"
    ).fetch_one(&state.db.pool).await.unwrap_or(0);

    let fish_avail: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fish_item WHERE sold_at IS NULL AND deleted_at IS NULL"
    ).fetch_one(&state.db.pool).await.unwrap_or(0);

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pending_sync WHERE synced_at IS NULL"
    ).fetch_one(&state.db.pool).await.unwrap_or(0);

    let top = sqlx::query_as::<_, TopProductRow>(
        "SELECT si.fish_type_name, COUNT(*) as quantity, COALESCE(SUM(si.subtotal), 0) as revenue
         FROM sale_item si JOIN sale s ON si.sale_id = s.id
         WHERE s.deleted_at IS NULL
         GROUP BY si.fish_type_name ORDER BY revenue DESC LIMIT ?1"
    )
    .bind(10)
    .fetch_all(&state.db.pool)
    .await.unwrap_or_default();

    let top_products: Vec<ProductInfo> = top.iter().map(|t| ProductInfo {
        product: t.fish_type_name.clone(),
        quantity: t.quantity,
        revenue: t.revenue.parse::<f64>().unwrap_or(0.0),
    }).collect();

    let pay = get_payment_summary(&state, None).await;
    let hourly = get_hourly_breakdown(&today_sales);

    let inv = get_inventory_summary(&state).await;

    let recent = sqlx::query_as::<_, SaleRow>(
        "SELECT id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
                subtotal, preparation_fee, total, item_count, created_at
         FROM sale WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db.pool)
    .await.unwrap_or_default();

    let recent_sales: Vec<serde_json::Value> = recent.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "customer_name": r.customer_name,
            "payment_method": r.payment_method_name,
            "total": r.total,
            "item_count": r.item_count,
            "created_at": r.created_at,
        })
    }).collect();

    Ok(Json(DashboardSummary {
        today_sales_count: today_sales.len() as i64,
        today_revenue: revenue,
        today_avg_ticket: if count > 0.0 { revenue / count } else { 0.0 },
        total_customers: customers,
        total_fish_types: fish_types,
        total_containers: containers,
        total_fish_available: fish_avail,
        pending_sync: pending,
        recent_sales,
        top_products,
        revenue_by_payment: pay,
        hourly_breakdown: hourly,
        inventory_summary: inv,
    }))
}

async fn get_today_sales(state: &AppState) -> Vec<SaleRow> {
    sqlx::query_as::<_, SaleRow>(
        "SELECT id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
                subtotal, preparation_fee, total, item_count, created_at
         FROM sale WHERE deleted_at IS NULL AND date(created_at) = date('now') ORDER BY created_at"
    )
    .fetch_all(&state.db.pool)
    .await.unwrap_or_default()
}

async fn get_payment_summary(state: &AppState, sale_ids: Option<&[String]>) -> Vec<PaymentInfo> {
    let sales: Vec<SaleRow> = if let Some(ids) = sale_ids {
        let mut vec = Vec::new();
        for id in ids {
            if let Ok(Some(row)) = sqlx::query_as::<_, SaleRow>(
                "SELECT id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
                        subtotal, preparation_fee, total, item_count, created_at
                 FROM sale WHERE id = ?1"
            ).bind(id).fetch_optional(&state.db.pool).await {
                vec.push(row);
            }
        }
        vec
    } else {
        sqlx::query_as::<_, SaleRow>(
            "SELECT id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
                    subtotal, preparation_fee, total, item_count, created_at
             FROM sale WHERE deleted_at IS NULL AND date(created_at) = date('now') ORDER BY created_at"
        )
        .fetch_all(&state.db.pool)
        .await.unwrap_or_default()
    };

    let mut map: HashMap<String, (i64, f64)> = HashMap::new();
    for s in &sales {
        let rev = s.total.parse::<f64>().unwrap_or(0.0);
        let e = map.entry(s.payment_method_name.clone()).or_insert((0, 0.0));
        e.0 += 1;
        e.1 += rev;
    }
    map.into_iter().map(|(method, (count, total))| PaymentInfo { method, count, total }).collect()
}

fn get_hourly_breakdown(sales: &[SaleRow]) -> Vec<HourlyInfo> {
    (0..24).map(|hour| {
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
        HourlyInfo { hour, sales: count, revenue: rev }
    }).collect()
}

async fn get_inventory_summary(state: &AppState) -> InventorySummary {
    let containers = sqlx::query_as::<_, ContainerRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count,
                location, is_active, op_counter
         FROM container WHERE is_active = 1 AND deleted_at IS NULL ORDER BY label"
    )
    .fetch_all(&state.db.pool)
    .await.unwrap_or_default();

    let mut containers_list: Vec<ContainerSummary> = Vec::new();
    let mut total_value = 0.0;
    let mut total_count = 0i64;

    for c in &containers {
        total_count += c.current_count as i64;
        let avg_weight: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(weight_grams), 0) FROM fish_item
             WHERE container_id = ?1 AND sold_at IS NULL AND deleted_at IS NULL"
        )
        .bind(&c.id)
        .fetch_one(&state.db.pool)
        .await.unwrap_or(0.0);

        let price: f64 = sqlx::query_scalar::<_, String>(
            "SELECT price_per_kg FROM market_price
             WHERE fish_type_id = ?1 AND effective_to IS NULL AND deleted_at IS NULL LIMIT 1"
        )
        .bind(&c.fish_type_id)
        .fetch_optional(&state.db.pool)
        .await.unwrap_or(None)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

        let pct = if c.capacity > 0 { c.current_count as f64 / c.capacity as f64 * 100.0 } else { 0.0 };
        let est_val = (avg_weight / 1000.0) * price * c.current_count as f64;
        total_value += est_val;

        containers_list.push(ContainerSummary {
            label: c.label.clone(),
            fish_type: c.fish_type_name.clone(),
            count: c.current_count,
            capacity: c.capacity,
            pct,
        });
    }

    InventorySummary {
        total_value,
        total_count,
        container_count: containers.len() as i64,
        containers: containers_list,
    }
}

pub async fn sales_trend(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TrendQuery>,
) -> ApiResult<Vec<TrendPoint>> {
    let days = query.days.unwrap_or(30).max(1).min(365);
    let period = query.period.as_deref().unwrap_or("day");

    let sql = match period {
        "week" => format!(
            "SELECT strftime('%Y-W%W', created_at) as label, COUNT(*) as sales, COALESCE(SUM(CAST(total AS REAL)), 0) as revenue
             FROM sale WHERE deleted_at IS NULL AND created_at >= date('now', '-{} days')
             GROUP BY label ORDER BY label", days
        ),
        "month" => format!(
            "SELECT strftime('%Y-%m', created_at) as label, COUNT(*) as sales, COALESCE(SUM(CAST(total AS REAL)), 0) as revenue
             FROM sale WHERE deleted_at IS NULL AND created_at >= date('now', '-{} days')
             GROUP BY label ORDER BY label", days
        ),
        _ => format!(
            "SELECT date(created_at) as label, COUNT(*) as sales, COALESCE(SUM(CAST(total AS REAL)), 0) as revenue
             FROM sale WHERE deleted_at IS NULL AND created_at >= date('now', '-{} days')
             GROUP BY date(created_at) ORDER BY label", days
        ),
    };

    let rows = sqlx::query_as::<_, TrendRow>(&sql)
        .fetch_all(&state.db.pool)
        .await?;

    let points: Vec<TrendPoint> = rows.iter().map(|r| TrendPoint {
        label: r.label.clone(),
        sales: r.sales,
        revenue: r.revenue,
    }).collect();

    Ok(Json(points))
}

// --- Top Products with date range ---

pub async fn top_products(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopQuery>,
) -> ApiResult<Vec<ProductInfo>> {
    let limit = query.limit.unwrap_or(10).max(1).min(100);

    let rows = sqlx::query_as::<_, TopProductRow>(
        "SELECT si.fish_type_name, COUNT(*) as quantity, COALESCE(SUM(CAST(si.subtotal AS REAL)), 0) as revenue
         FROM sale_item si JOIN sale s ON si.sale_id = s.id
         WHERE s.deleted_at IS NULL
         GROUP BY si.fish_type_name ORDER BY revenue DESC LIMIT ?1"
    )
    .bind(limit)
    .fetch_all(&state.db.pool)
    .await?;

    let products: Vec<ProductInfo> = rows.iter().map(|r| ProductInfo {
        product: r.fish_type_name.clone(),
        quantity: r.quantity,
        revenue: r.revenue.parse::<f64>().unwrap_or(0.0),
    }).collect();

    Ok(Json(products))
}

// --- Row types ---

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
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct TopProductRow {
    fish_type_name: String,
    quantity: i64,
    revenue: String,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct TrendRow {
    label: String,
    sales: i64,
    revenue: f64,
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
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct PaymentSummaryRow {
    payment_method_name: String,
    count: i64,
    total: String,
}
