use axum::{extract::{Path, State}, Json};
use std::sync::Arc;
use crate::AppState;

pub async fn overview(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let branch_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM branches WHERE deleted_at IS NULL AND is_active = true")
        .fetch_one(&state.db.pool)
        .await
        .unwrap_or(0);

    let total_sales: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sales WHERE deleted_at IS NULL AND created_at >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&state.db.pool)
    .await
    .unwrap_or(0);

    let total_revenue: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total), 0) FROM sales WHERE deleted_at IS NULL AND created_at >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&state.db.pool)
    .await
    .unwrap_or(0.0);

    Json(serde_json::json!({
        "branch_count": branch_count,
        "total_sales_24h": total_sales,
        "total_revenue_24h": total_revenue,
        "status": "online",
    }))
}

pub async fn list_branches(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<serde_json::Value>> {
    let rows = sqlx::query_as::<_, (String, String, String, String, bool)>(
        "SELECT id, name, address, phone, is_active FROM branches WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default();

    let branches: Vec<serde_json::Value> = rows.into_iter().map(|(id, name, address, phone, active)| {
        serde_json::json!({
            "id": id,
            "name": name,
            "address": address,
            "phone": phone,
            "is_active": active,
        })
    }).collect();

    Json(branches)
}

pub async fn branch_sales(
    State(state): State<Arc<AppState>>,
    Path(branch_id): Path<String>,
) -> Json<serde_json::Value> {
    let sales = sqlx::query_as::<_, (String, String, String, String, String, i32)>(
        "SELECT id, payment_method_name, total::text, subtotal::text, created_at::text, item_count
         FROM sales WHERE branch_id = $1 AND deleted_at IS NULL
         ORDER BY created_at DESC LIMIT 50"
    )
    .bind(&branch_id)
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = sales.into_iter().map(|(id, pm, total, subtotal, created_at, count)| {
        serde_json::json!({
            "id": id,
            "payment_method": pm,
            "total": total,
            "subtotal": subtotal,
            "item_count": count,
            "created_at": created_at,
        })
    }).collect();

    Json(serde_json::json!({ "sales": items }))
}
