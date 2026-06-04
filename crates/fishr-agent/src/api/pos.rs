use axum::extract::State;
use axum::Json;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Timelike, Datelike};
use fishr_core::models::*;
use fishr_core::fuzzy::sets::*;
use fishr_core::fuzzy::suggestions::*;
use fishr_core::aco::graph::{PrepGraph, PrepNode};
use fishr_core::aco::engine::{AcoConfig, AcoSolver};
use crate::api::error::{ApiResult, ApiError, validate_not_empty, validate_weight};
use crate::state::AppState;

#[derive(Serialize)]
pub struct CalculatedSale {
    pub items: Vec<CalculatedItem>,
    pub subtotal: f64,
    pub preparation_fee: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub iva_rate: f64,
    pub total: f64,
    pub preparation_sequence: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct CalculatedItem {
    pub fish_item_id: String,
    pub container_label: String,
    pub fish_type_name: String,
    pub weight_grams: i32,
    pub weight_kg: f64,
    pub price_per_kg: f64,
    pub preparation_name: Option<String>,
    pub preparation_fee: f64,
    pub subtotal: f64,
    pub preparation_order: Option<usize>,
}

#[derive(Deserialize)]
pub struct CalculateRequest {
    pub items: Vec<CalculateItemRequest>,
    pub iva_rate: Option<f64>,
    pub discount: Option<f64>,
}

#[derive(Deserialize)]
pub struct SuggestionsRequest {
    pub items: Vec<CalculateItemRequest>,
    pub customer_id: Option<String>,
}

#[derive(Serialize)]
pub struct SuggestionsResponse {
    pub suggestions: Vec<SuggestionOutput>,
}

#[derive(Serialize)]
pub struct SuggestionOutput {
    pub r#type: String,
    pub message: String,
    pub reason: String,
    pub confidence: f64,
    pub preparation_id: Option<String>,
    pub max_discount_pct: Option<f64>,
}

#[derive(Deserialize, Clone)]
pub struct CalculateItemRequest {
    pub fish_item_id: String,
    pub preparation_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfirmSaleRequest {
    pub customer_id: Option<String>,
    pub payment_method_id: String,
    pub items: Vec<ConfirmSaleItem>,
    pub iva_rate: Option<f64>,
    pub discount: Option<f64>,
    pub tax_amount: Option<f64>,
    pub discount_amount: Option<f64>,
}

#[derive(Deserialize, Clone)]
pub struct ConfirmSaleItem {
    pub fish_item_id: String,
    pub container_id: String,
    pub weight_grams: i32,
    #[allow(dead_code)]
    pub price_per_kg: f64,
    pub preparation_id: Option<String>,
    pub preparation_name: Option<String>,
    pub preparation_fee: f64,
}

pub async fn list_payment_methods(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Vec<PaymentMethod>> {
    let rows = sqlx::query_as::<_, PaymentMethodRow>(
        "SELECT id, branch_id, name, description, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM payment_method WHERE is_active = 1 AND deleted_at IS NULL"
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

pub async fn list_preparations(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Vec<Preparation>> {
    let rows = sqlx::query_as::<_, PreparationRow>(
        "SELECT id, branch_id, name, description, additional_cost, cost_type, is_active,
                op_counter, updated_at, synced_at, deleted_at
         FROM preparation WHERE is_active = 1 AND deleted_at IS NULL"
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

async fn precompute_price_factors(state: &AppState) -> anyhow::Result<std::collections::HashMap<String, f64>> {
    let now = chrono::Utc::now();
    let hour = now.hour() as f64;

    // Stock per fish type: aggregate from containers
    let stock_rows = sqlx::query_as::<_, StockByType>(
        "SELECT fish_type_id, SUM(current_count) as total_count, SUM(capacity) as total_capacity
         FROM container WHERE deleted_at IS NULL AND is_active = 1
         GROUP BY fish_type_id"
    )
    .fetch_all(&state.db.pool)
    .await?;

    let mut stock_by_type: std::collections::HashMap<String, (i64, i64)> = std::collections::HashMap::new();
    for r in &stock_rows {
        stock_by_type.insert(r.fish_type_id.clone(), (r.total_count, r.total_capacity));
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

    let engine = build_pricing_engine();
    let month = now.month();
    let mut factors = std::collections::HashMap::new();

    let prices = sqlx::query_as::<_, MarketPriceRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, price_per_kg, cost_price,
                effective_from, effective_to, op_counter, updated_at, synced_at, deleted_at
         FROM market_price WHERE effective_to IS NULL AND deleted_at IS NULL"
    )
    .fetch_all(&state.db.pool)
    .await?;

    for price in &prices {
        let (total_count, total_cap) = stock_by_type.get(&price.fish_type_id).copied().unwrap_or((0, 0));
        let stock_pct = if total_cap > 0 { (total_count as f64 / total_cap as f64) * 100.0 } else { 50.0 };

        let days_since = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&price.effective_from) {
            let dt_utc = dt.with_timezone(&chrono::Utc);
            (now - dt_utc).num_days() as f64
        } else { 0.0 };

        let fish_sales: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sale_item WHERE fish_type_id = ?1 AND updated_at >= ?2"
        )
        .bind(&price.fish_type_id)
        .bind(&week_ago)
        .fetch_one(&state.db.pool)
        .await
        .unwrap_or(0);
        let popularity_pct = (fish_sales as f64 / total_items_sold as f64) * 100.0;

        let seasonal_demand_pct = match month {
            12 | 1 | 2 => 70.0,
            3 | 4 => 50.0,
            5 | 6 | 7 => 40.0,
            8 | 9 => 55.0,
            10 | 11 => 60.0,
            _ => 50.0,
        };

        let input = FuzzyInput {
            stock_pct, hour,
            popularity_pct, customer_visits_pct: 0.0, hourly_demand_pct,
            days_since_price_change: days_since.min(30.0),
            seasonal_demand_pct,
        };
        let (factor, _) = compute_price_factor(&engine, &input);
        factors.insert(price.fish_type_id.clone(), factor);
    }

    Ok(factors)
}

pub async fn calculate_sale(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CalculateRequest>,
) -> ApiResult<CalculatedSale> {
    let price_factors = precompute_price_factors(&state).await.unwrap_or_default();
    let mut items = Vec::new();
    let mut subtotal_total = 0.0f64;
    let mut prep_total = 0.0f64;

    for item in &req.items {
        let fish = sqlx::query_as::<_, FishItemRow>(
            "SELECT id, branch_id, container_id, container_label, fish_type_id, fish_type_name,
                    weight_grams, added_at, sold_at, sold_in_sale_id,
                    op_counter, updated_at, synced_at, deleted_at
             FROM fish_item WHERE id = ?1"
        )
        .bind(&item.fish_item_id)
        .fetch_one(&state.db.pool)
        .await?;

        let price = sqlx::query_as::<_, MarketPriceRow>(
            "SELECT id, branch_id, fish_type_id, fish_type_name, price_per_kg, cost_price,
                    effective_from, effective_to, op_counter, updated_at, synced_at, deleted_at
             FROM market_price WHERE fish_type_id = ?1 AND effective_to IS NULL AND deleted_at IS NULL"
        )
        .bind(&fish.fish_type_id)
        .fetch_optional(&state.db.pool)
        .await?;

        let base_price = price.as_ref()
            .map(|p| p.price_per_kg)
            .ok_or_else(|| ApiError::bad_request(format!("No hay precio de mercado para {}", fish.fish_type_name)))?;

        // Apply dynamic pricing factor
        let dyn_factor = price_factors.get(&fish.fish_type_id).copied().unwrap_or(1.0);
        let price_per_kg = base_price * dyn_factor;

        let (_prep_row, prep_name, prep_fee) = if let Some(ref pid) = item.preparation_id {
            let prep = sqlx::query_as::<_, PreparationRow>(
                "SELECT id, branch_id, name, description, additional_cost, cost_type, is_active,
                        op_counter, updated_at, synced_at, deleted_at
                 FROM preparation WHERE id = ?1"
            )
            .bind(pid)
            .fetch_optional(&state.db.pool)
            .await?;

            match prep {
                Some(p) => {
                    let weight_kg = fish.weight_grams as f64 / 1000.0;
                    let base = weight_kg * price_per_kg;
                    let cost_val = p.additional_cost.parse::<f64>().unwrap_or(0.0);
                    let fee = match p.cost_type.as_str() {
                        "Percentage" => base * cost_val / 100.0,
                        _ => cost_val,
                    };
                    (Some(p.name.clone()), Some(p.name), fee)
                }
                None => (None, None, 0.0),
            }
        } else {
            (None, None, 0.0)
        };

        let weight_kg = fish.weight_grams as f64 / 1000.0;
        let subtotal = weight_kg * price_per_kg;
        subtotal_total += subtotal;
        prep_total += prep_fee;

        items.push(CalculatedItem {
            fish_item_id: item.fish_item_id.clone(),
            container_label: fish.container_label,
            fish_type_name: fish.fish_type_name,
            weight_grams: fish.weight_grams,
            weight_kg,
            price_per_kg,
            preparation_name: prep_name,
            preparation_fee: prep_fee,
            subtotal,
            preparation_order: None,
        });
    }

    let iva_rate = req.iva_rate.unwrap_or(16.0).max(0.0).min(100.0);
    let discount = req.discount.unwrap_or(0.0).max(0.0);
    let base = subtotal_total + prep_total;
    let taxable_base = (base - discount).max(0.0);
    let tax_amount = taxable_base * iva_rate / 100.0;
    let total = taxable_base + tax_amount;

    // ACO-based preparation sequence optimization
    let preparation_sequence = optimize_prep_sequence(&items);
    let items = apply_prep_sequence(items, &preparation_sequence);

    Ok(Json(CalculatedSale {
        total,
        subtotal: subtotal_total,
        preparation_fee: prep_total,
        discount_amount: discount,
        tax_amount,
        iva_rate,
        items,
        preparation_sequence,
    }))
}

pub async fn confirm_sale(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfirmSaleRequest>,
) -> ApiResult<serde_json::Value> {
    validate_not_empty(&req.payment_method_id, "método de pago")?;
    if req.items.is_empty() {
        return Err(ApiError::bad_request("La venta debe tener al menos un item"));
    }

    let now = chrono::Utc::now();
    let price_factors = precompute_price_factors(&state).await.unwrap_or_default();

    let pm = sqlx::query_as::<_, PaymentMethodRow>(
        "SELECT id, branch_id, name, description, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM payment_method WHERE id = ?1"
    )
    .bind(&req.payment_method_id)
    .fetch_one(&state.db.pool)
    .await?;

    let customer_name = match &req.customer_id {
        Some(cid) => {
            sqlx::query_scalar::<_, String>("SELECT name FROM customer WHERE id = ?1")
                .bind(cid)
                .fetch_optional(&state.db.pool)
                .await?
        }
        None => None,
    };

    // Create sale items and compute totals (prices reconciled server-side)
    let sale_id = ulid::Ulid::new().to_string();
    let mut sale_items = Vec::new();
    for item in &req.items {
        validate_not_empty(&item.fish_item_id, "item de pescado")?;
        validate_not_empty(&item.container_id, "contenedor")?;
        validate_weight(item.weight_grams)?;

        let fish = sqlx::query_as::<_, FishItemRow>(
            "SELECT id, branch_id, container_id, container_label, fish_type_id, fish_type_name,
                    weight_grams, added_at, sold_at, sold_in_sale_id,
                    op_counter, updated_at, synced_at, deleted_at
             FROM fish_item WHERE id = ?1"
        )
        .bind(&item.fish_item_id)
        .fetch_one(&state.db.pool)
        .await?;

        // Reconcile price server-side: fetch current market price
        let market_price = sqlx::query_as::<_, MarketPriceRow>(
            "SELECT id, branch_id, fish_type_id, fish_type_name, price_per_kg, cost_price,
                    effective_from, effective_to, op_counter, updated_at, synced_at, deleted_at
             FROM market_price WHERE fish_type_id = ?1 AND effective_to IS NULL AND deleted_at IS NULL"
        )
        .bind(&fish.fish_type_id)
        .fetch_optional(&state.db.pool)
        .await?;

        let base_price = market_price
            .as_ref()
            .map(|p| p.price_per_kg)
            .ok_or_else(|| ApiError::bad_request(format!("No hay precio de mercado para {}", fish.fish_type_name)))?;

        let dyn_factor = price_factors.get(&fish.fish_type_id).copied().unwrap_or(1.0);
        let adjusted_price = base_price * dyn_factor;

        let pkg = rust_decimal::Decimal::from_f64_retain(adjusted_price)
            .ok_or_else(|| ApiError::bad_request("precio por kg inválido"))?;
        let pf = rust_decimal::Decimal::from_f64_retain(item.preparation_fee)
            .unwrap_or_default();

        sale_items.push(SaleItem::new(
            state.config.branch_id.clone(),
            sale_id.clone(),
            item.fish_item_id.clone(),
            item.container_id.clone(),
            fish.container_label,
            fish.fish_type_id,
            fish.fish_type_name,
            item.weight_grams,
            pkg,
            item.preparation_id.clone(),
            item.preparation_name.clone(),
            pf,
        ));
    }

    // Compute sale totals with IVA and discount
    let iva_rate = req.iva_rate.unwrap_or(16.0).max(0.0).min(100.0);
    let discount_amount_val = req.discount_amount.unwrap_or(req.discount.unwrap_or(0.0)).max(0.0);
    let tax_amount_val = req.tax_amount.unwrap_or_else(|| {
        let base: f64 = sale_items.iter().map(|i| {
            let sub = i.subtotal.to_f64().unwrap_or(0.0);
            let prep = i.preparation_fee.to_f64().unwrap_or(0.0);
            sub + prep
        }).sum::<f64>();
        let taxable = (base - discount_amount_val).max(0.0);
        taxable * iva_rate / 100.0
    });

    let sale_total = sale_items.iter().map(|i| {
        let sub = i.subtotal.to_f64().unwrap_or(0.0);
        let prep = i.preparation_fee.to_f64().unwrap_or(0.0);
        sub + prep
    }).sum::<f64>() - discount_amount_val + tax_amount_val;

    let sale = Sale::with_id(
        sale_id.clone(),
        state.config.branch_id.clone(),
        req.customer_id.clone(),
        customer_name.clone(),
        req.payment_method_id,
        pm.name,
        &sale_items,
    );

    // Insert sale with IVA and discount
    sqlx::query(
        "INSERT INTO sale (id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
         subtotal, preparation_fee, tax_amount, discount_amount, iva_rate, total, item_count, created_at, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
    )
    .bind(&sale.id)
    .bind(&sale.branch_id)
    .bind(&sale.customer_id)
    .bind(&sale.customer_name)
    .bind(&sale.payment_method_id)
    .bind(&sale.payment_method_name)
    .bind(sale.subtotal.to_string())
    .bind(sale.preparation_fee.to_string())
    .bind(tax_amount_val.to_string())
    .bind(discount_amount_val.to_string())
    .bind(iva_rate)
    .bind(sale_total.to_string())
    .bind(sale.item_count)
    .bind(sale.created_at)
    .bind(sale.op_counter)
    .bind(sale.updated_at)
    .execute(&state.db.pool)
    .await?;

    // Insert sale items and mark fish as sold
    for item in &sale_items {
        sqlx::query(
            "INSERT INTO sale_item (id, branch_id, sale_id, fish_item_id, container_id, container_label,
             fish_type_id, fish_type_name, weight_grams, price_per_kg, preparation_id, preparation_name,
             preparation_fee, subtotal, op_counter, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
        )
        .bind(&item.id)
        .bind(&item.branch_id)
        .bind(&item.sale_id)
        .bind(&item.fish_item_id)
        .bind(&item.container_id)
        .bind(&item.container_label)
        .bind(&item.fish_type_id)
        .bind(&item.fish_type_name)
        .bind(item.weight_grams)
        .bind(item.price_per_kg.to_string())
        .bind(&item.preparation_id)
        .bind(&item.preparation_name)
        .bind(item.preparation_fee.to_string())
        .bind(item.subtotal.to_string())
        .bind(item.op_counter)
        .bind(item.updated_at)
        .execute(&state.db.pool)
        .await?;

        sqlx::query("UPDATE fish_item SET sold_at=?1, sold_in_sale_id=?2, updated_at=?3, op_counter=?4 WHERE id=?5")
            .bind(sale.created_at)
            .bind(&sale.id)
            .bind(now)
            .bind(now.timestamp_millis())
            .bind(&item.fish_item_id)
            .execute(&state.db.pool)
            .await?;
    }

    // Update container counts
    for item in &req.items {
        sqlx::query("UPDATE container SET current_count = current_count - 1 WHERE id = ?1 AND current_count > 0")
            .bind(&item.container_id)
            .execute(&state.db.pool)
            .await?;
    }

    // Update customer points
    if let Some(cid) = &req.customer_id {
        sqlx::query("UPDATE customer SET points = points + ?1 WHERE id = ?2")
            .bind(sale.total.to_string())
            .bind(cid)
            .execute(&state.db.pool)
            .await?;
    }

    // Generate invoice control number
    let rev_chars: String = ulid::Ulid::new().to_string().chars().rev().collect();
    let suffix = &rev_chars[..rev_chars.len().min(8)];
    let control = format!(
        "F-{:04}-{:08}",
        chrono::Utc::now().format("%Y%m"),
        suffix
    );

    let invoice = Invoice::new(
        state.config.branch_id.clone(),
        sale.id.clone(),
        req.customer_id,
        customer_name,
        None,
        None,
        control,
        rust_decimal::Decimal::from_f64_retain(sale_total).unwrap_or_default(),
    );

    sqlx::query(
        "INSERT INTO invoice (id, branch_id, sale_id, customer_id, customer_name, customer_rif, customer_address,
         control_number, total, tax_amount, issued_at, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
    )
    .bind(&invoice.id)
    .bind(&invoice.branch_id)
    .bind(&invoice.sale_id)
    .bind(&invoice.customer_id)
    .bind(&invoice.customer_name)
    .bind(&invoice.customer_rif)
    .bind(&invoice.customer_address)
    .bind(&invoice.control_number)
    .bind(invoice.total.to_string())
    .bind(tax_amount_val.to_string())
    .bind(invoice.issued_at)
    .bind(invoice.op_counter)
    .bind(invoice.updated_at)
    .execute(&state.db.pool)
    .await?;

    crate::api::inventory::push_sync(&state, "Sale", &sale).await;
    for item in &sale_items {
        crate::api::inventory::push_sync(&state, "SaleItem", item).await;
    }
    crate::api::inventory::push_sync(&state, "Invoice", &invoice).await;

    Ok(Json(serde_json::json!({
        "sale_id": sale.id,
        "total": sale.total.to_string(),
        "invoice": invoice.control_number,
        "item_count": sale.item_count,
        "created_at": sale.created_at,
    })))
}

pub async fn sale_suggestions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SuggestionsRequest>,
) -> ApiResult<SuggestionsResponse> {
    let now = chrono::Utc::now();
    let hour = now.hour() as f64;

    // Get overall stock percentage across all containers
    let stock_info = sqlx::query_as::<_, StockInfo>(
        "SELECT COALESCE(SUM(current_count), 0) as total_count, COALESCE(SUM(capacity), 0) as total_capacity
         FROM container WHERE deleted_at IS NULL"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let stock_pct = if stock_info.total_capacity > 0 {
        (stock_info.total_count as f64 / stock_info.total_capacity as f64) * 100.0
    } else {
        50.0
    };

    // Get today's total sales count for demand calculation
    let today_start = now.format("%Y-%m-%dT00:00:00").to_string();
    let today_sales: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sale WHERE created_at >= ?1"
    )
    .bind(&today_start)
    .fetch_one(&state.db.pool)
    .await
    .unwrap_or(0);

    let hourly_demand_pct = (today_sales as f64 / 24.0).min(100.0);

    // Get popularity: total sales per fish type (last 7 days)
    let week_ago = (now - chrono::Duration::days(7)).format("%Y-%m-%dT%H:%M:%S").to_string();
    let total_items_sold: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sale_item WHERE updated_at >= ?1"
    )
    .bind(&week_ago)
    .fetch_one(&state.db.pool)
    .await
    .unwrap_or(1).max(1);

    let engine = build_pos_engine();

    // Get customer loyalty if provided
    let customer_visits_pct = if let Some(cid) = &req.customer_id {
        let visits: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sale WHERE customer_id = ?1"
        )
        .bind(cid)
        .fetch_one(&state.db.pool)
        .await
        .unwrap_or(0);
        (visits as f64 / 50.0).min(100.0) * 100.0
    } else {
        0.0
    };

    let mut all_suggestions = Vec::new();

    // Get all preparations for matching
    let prep_rows = sqlx::query_as::<_, PreparationRow>(
        "SELECT id, branch_id, name, description, additional_cost, cost_type, is_active,
                op_counter, updated_at, synced_at, deleted_at
         FROM preparation WHERE is_active = 1 AND deleted_at IS NULL"
    )
    .fetch_all(&state.db.pool)
    .await?;

    let preparations: Vec<(String, String)> = prep_rows.iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();
    let prep_refs: Vec<(&str, &str)> = preparations.iter()
        .map(|(id, name)| (id.as_str(), name.as_str()))
        .collect();

    for item in &req.items {
        let fish = sqlx::query_as::<_, FishItemRow>(
            "SELECT id, branch_id, container_id, container_label, fish_type_id, fish_type_name,
                    weight_grams, added_at, sold_at, sold_in_sale_id,
                    op_counter, updated_at, synced_at, deleted_at
             FROM fish_item WHERE id = ?1"
        )
        .bind(&item.fish_item_id)
        .fetch_optional(&state.db.pool)
        .await?;

        let (fish_type_id, fish_type_name, category) = match fish {
            Some(ref f) => {
                let cat: Option<String> = sqlx::query_scalar(
                    "SELECT category FROM fish_type WHERE id = ?1"
                )
                .bind(&f.fish_type_id)
                .fetch_optional(&state.db.pool)
                .await
                .unwrap_or(None);
                (f.fish_type_id.clone(), f.fish_type_name.clone(), cat.unwrap_or_default())
            }
            None => continue,
        };

        // Fish type popularity
        let fish_sales: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sale_item WHERE fish_type_id = ?1 AND updated_at >= ?2"
        )
        .bind(&fish_type_id)
        .bind(&week_ago)
        .fetch_one(&state.db.pool)
        .await
        .unwrap_or(0);

        let popularity_pct = (fish_sales as f64 / total_items_sold as f64) * 100.0;

        let input = FuzzyInput::new_pos(
            stock_pct,
            hour,
            popularity_pct,
            customer_visits_pct,
            hourly_demand_pct,
        );

        let raw = engine.evaluate(&input);
        let matched = match_preparation_suggestion(&raw, &fish_type_name, &category, &prep_refs);

        for s in matched {
            let (stype, message, reason, prep_id, discount_pct) = match &s.suggestion_type {
                SuggestionType::SuggestDiscount { max_discount_pct, reason } => {
                    ("discount", format!("Descuento hasta {}%", max_discount_pct), reason.clone(), None, Some(*max_discount_pct))
                }
                SuggestionType::SuggestPromotion { message, reason } => {
                    ("promotion", message.clone(), reason.clone(), None, None)
                }
                SuggestionType::SuggestPreparation { preparation_id, reason } => {
                    ("preparation", "¿Agregar preparación?".into(), reason.clone(), Some(preparation_id.clone()), None)
                }
                SuggestionType::SuggestUpsell { message, reason } => {
                    ("upsell", message.clone(), reason.clone(), None, None)
                }
                SuggestionType::PriceFactor { factor, reason } => {
                    ("price_factor", format!("Factor sugerido: {:.0}%", factor * 100.0), reason.clone(), None, None)
                }
            };
            all_suggestions.push(SuggestionOutput {
                r#type: stype.to_string(),
                message,
                reason,
                confidence: s.confidence,
                preparation_id: prep_id,
                max_discount_pct: discount_pct,
            });
        }
    }

    // Deduplicate by type+message, keep highest confidence
    all_suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    let mut seen = std::collections::HashSet::new();
    all_suggestions.retain(|s| {
        seen.insert((s.r#type.clone(), s.message.clone()))
    });

    // Keep top 5
    all_suggestions.truncate(5);

    Ok(Json(SuggestionsResponse {
        suggestions: all_suggestions,
    }))
}

// Row types
#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct StockByType {
    fish_type_id: String,
    total_count: i64,
    total_capacity: i64,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct StockInfo {
    total_count: i64,
    total_capacity: i64,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct FishItemRow {
    id: String,
    branch_id: String,
    container_id: String,
    container_label: String,
    fish_type_id: String,
    fish_type_name: String,
    weight_grams: i32,
    added_at: String,
    sold_at: Option<String>,
    sold_in_sale_id: Option<String>,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct MarketPriceRow {
    id: String,
    branch_id: String,
    fish_type_id: String,
    fish_type_name: String,
    price_per_kg: f64,
    cost_price: f64,
    effective_from: String,
    effective_to: Option<String>,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PaymentMethodRow {
    id: String,
    branch_id: String,
    name: String,
    description: String,
    is_active: bool,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

impl PaymentMethodRow {
    fn into_model(self) -> PaymentMethod {
        PaymentMethod {
            id: self.id,
            branch_id: self.branch_id,
            name: self.name,
            description: self.description,
            is_active: self.is_active,
            op_counter: self.op_counter,
            updated_at: self.updated_at.parse().unwrap_or_default(),
            synced_at: self.synced_at.and_then(|s| s.parse().ok()),
            deleted_at: self.deleted_at.and_then(|s| s.parse().ok()),
        }
    }
}

#[derive(sqlx::FromRow)]
struct PreparationRow {
    id: String,
    branch_id: String,
    name: String,
    description: String,
    additional_cost: String,
    cost_type: String,
    is_active: bool,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

impl PreparationRow {
    fn into_model(self) -> Preparation {
        Preparation {
            id: self.id,
            branch_id: self.branch_id,
            name: self.name,
            description: self.description,
            additional_cost: self.additional_cost.parse().unwrap_or_default(),
            cost_type: if self.cost_type == "Percentage" { fishr_core::models::CostType::Percentage } else { fishr_core::models::CostType::Fixed },
            is_active: self.is_active,
            op_counter: self.op_counter,
            updated_at: self.updated_at.parse().unwrap_or_default(),
            synced_at: self.synced_at.and_then(|s| s.parse().ok()),
            deleted_at: self.deleted_at.and_then(|s| s.parse().ok()),
        }
    }
}

fn optimize_prep_sequence(items: &[CalculatedItem]) -> Option<Vec<String>> {
    let prep_items: Vec<&CalculatedItem> = items.iter()
        .filter(|i| i.preparation_name.is_some())
        .collect();

    if prep_items.len() < 2 {
        return None;
    }

    let nodes: Vec<PrepNode> = prep_items.iter().enumerate().map(|(idx, item)| {
        PrepNode {
            index: idx,
            fish_item_id: item.fish_item_id.clone(),
            fish_type_name: item.fish_type_name.clone(),
            preparation_id: item.preparation_name.clone().unwrap_or_default(),
            preparation_name: item.preparation_name.clone().unwrap_or_default(),
            category: String::new(),
        }
    }).collect();

    let graph = PrepGraph::new(nodes.clone());
    let config = AcoConfig::default();
    let solver = AcoSolver::new(config);
    let result = solver.solve(&graph);

    let sequence: Vec<String> = result.best_path.order.iter()
        .map(|&i| nodes[i].fish_item_id.clone())
        .collect();

    Some(sequence)
}

fn apply_prep_sequence(mut items: Vec<CalculatedItem>, sequence: &Option<Vec<String>>) -> Vec<CalculatedItem> {
    let seq = match sequence {
        Some(s) => s,
        None => return items,
    };

    let mut order_map: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (pos, id) in seq.iter().enumerate() {
        order_map.insert(id.as_str(), pos + 1);
    }

    for item in &mut items {
        item.preparation_order = order_map.get(item.fish_item_id.as_str()).copied();
    }

    items
}
