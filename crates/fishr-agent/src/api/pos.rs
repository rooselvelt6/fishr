use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use fishr_core::models::*;
use crate::api::error::{ApiResult, ApiError, validate_not_empty, validate_weight};
use crate::state::AppState;

#[derive(Serialize)]
pub struct CalculatedSale {
    pub items: Vec<CalculatedItem>,
    pub subtotal: f64,
    pub preparation_fee: f64,
    pub total: f64,
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
}

#[derive(Deserialize)]
pub struct CalculateRequest {
    pub items: Vec<CalculateItemRequest>,
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

pub async fn calculate_sale(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CalculateRequest>,
) -> ApiResult<CalculatedSale> {
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

        let price_per_kg = price.as_ref()
            .map(|p| p.price_per_kg)
            .ok_or_else(|| ApiError::bad_request(format!("No hay precio de mercado para {}", fish.fish_type_name)))?;

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
        });
    }

    Ok(Json(CalculatedSale {
        total: subtotal_total + prep_total,
        subtotal: subtotal_total,
        preparation_fee: prep_total,
        items,
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

        let price_per_kg = market_price
            .as_ref()
            .map(|p| p.price_per_kg)
            .ok_or_else(|| ApiError::bad_request(format!("No hay precio de mercado para {}", fish.fish_type_name)))?;

        let pkg = rust_decimal::Decimal::from_f64_retain(price_per_kg)
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

    let sale = Sale::with_id(
        sale_id.clone(),
        state.config.branch_id.clone(),
        req.customer_id.clone(),
        customer_name.clone(),
        req.payment_method_id,
        pm.name,
        &sale_items,
    );

    // Insert sale
    sqlx::query(
        "INSERT INTO sale (id, branch_id, customer_id, customer_name, payment_method_id, payment_method_name,
         subtotal, preparation_fee, total, item_count, created_at, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
    )
    .bind(&sale.id)
    .bind(&sale.branch_id)
    .bind(&sale.customer_id)
    .bind(&sale.customer_name)
    .bind(&sale.payment_method_id)
    .bind(&sale.payment_method_name)
    .bind(sale.subtotal.to_string())
    .bind(sale.preparation_fee.to_string())
    .bind(sale.total.to_string())
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
        sale.total,
    );

    sqlx::query(
        "INSERT INTO invoice (id, branch_id, sale_id, customer_id, customer_name, customer_rif, customer_address,
         control_number, total, issued_at, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
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

// Row types
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
