use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use fishr_core::models::*;
use crate::api::error::{ApiResult, ApiError, validate_not_empty, validate_positive_i32, validate_weight, validate_non_negative_f64};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub rif: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub address: Option<String>,
    pub contact_person: Option<String>,
    pub is_self: Option<bool>,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_suppliers(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Vec<Supplier>> {
    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);
    let rows = sqlx::query_as::<_, SupplierRow>(
        "SELECT id, branch_id, name, rif, phone, email, address, contact_person,
                is_self, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM supplier WHERE deleted_at IS NULL ORDER BY name LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

pub async fn create_supplier(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSupplierRequest>,
) -> ApiResult<Supplier> {
    validate_not_empty(&req.name, "nombre")?;
    validate_not_empty(&req.phone, "teléfono")?;

    let mut s = Supplier::new(
        state.config.branch_id.clone(),
        req.name,
        req.phone,
    );
    s.rif = req.rif;
    s.email = req.email;
    s.address = req.address;
    s.contact_person = req.contact_person.unwrap_or_default();
    s.is_self = req.is_self.unwrap_or(false);

    sqlx::query(
        "INSERT INTO supplier (id, branch_id, name, rif, phone, email, address, contact_person,
         is_self, is_active, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)"
    )
    .bind(&s.id)
    .bind(&s.branch_id)
    .bind(&s.name)
    .bind(&s.rif)
    .bind(&s.phone)
    .bind(&s.email)
    .bind(&s.address)
    .bind(&s.contact_person)
    .bind(s.is_self)
    .bind(s.op_counter)
    .bind(s.updated_at)
    .execute(&state.db.pool)
    .await?;

    crate::sync::push_sync(&state, "Supplier", &s).await;
    Ok(Json(s))
}

pub async fn get_supplier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Option<Supplier>> {
    let row = sqlx::query_as::<_, SupplierRow>(
        "SELECT id, branch_id, name, rif, phone, email, address, contact_person,
                is_self, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM supplier WHERE id = ?1 AND deleted_at IS NULL"
    )
    .bind(&id)
    .fetch_optional(&state.db.pool)
    .await?;

    Ok(Json(row.map(|r| r.into_model())))
}

pub async fn update_supplier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CreateSupplierRequest>,
) -> ApiResult<Supplier> {
    validate_not_empty(&req.name, "nombre")?;
    validate_not_empty(&req.phone, "teléfono")?;

    let now = chrono::Utc::now();
    sqlx::query(
        "UPDATE supplier SET name=?1, rif=?2, phone=?3, email=?4, address=?5,
         contact_person=?6, is_self=?7, updated_at=?8, op_counter=?9 WHERE id=?10"
    )
    .bind(&req.name)
    .bind(&req.rif)
    .bind(&req.phone)
    .bind(&req.email)
    .bind(&req.address)
    .bind(&req.contact_person)
    .bind(req.is_self.unwrap_or(false))
    .bind(now)
    .bind(now.timestamp_millis())
    .bind(&id)
    .execute(&state.db.pool)
    .await?;

    let row = sqlx::query_as::<_, SupplierRow>(
        "SELECT id, branch_id, name, rif, phone, email, address, contact_person,
                is_self, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM supplier WHERE id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db.pool)
    .await?;

    let s = row.into_model();
    crate::sync::push_sync(&state, "Supplier", &s).await;
    Ok(Json(s))
}

pub async fn delete_supplier(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<&'static str> {
    let now = chrono::Utc::now();
    sqlx::query("UPDATE supplier SET deleted_at=?1, updated_at=?2, op_counter=?3 WHERE id=?4")
        .bind(now)
        .bind(now)
        .bind(now.timestamp_millis())
        .bind(&id)
        .execute(&state.db.pool)
        .await?;

    Ok(Json("Eliminado"))
}

// --- Deliveries ---

#[derive(Deserialize)]
pub struct CreateDeliveryRequest {
    pub supplier_id: String,
    pub transport_plate: Option<String>,
    pub transport_driver: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<DeliveryItemRequest>,
}

#[derive(Deserialize)]
pub struct DeliveryItemRequest {
    pub container_id: String,
    pub fish_type_id: String,
    pub quantity: i32,
    pub weight_grams: i32,
    pub unit_cost: f64,
}

#[derive(Serialize)]
pub struct DeliveryResponse {
    pub delivery: SupplierDelivery,
    pub items: Vec<SupplierDeliveryItem>,
    pub fish_created: i32,
}

pub async fn create_delivery(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDeliveryRequest>,
) -> ApiResult<DeliveryResponse> {
    validate_not_empty(&req.supplier_id, "proveedor")?;
    if req.items.is_empty() {
        return Err(ApiError::bad_request("La entrega debe tener al menos un item"));
    }

    let supp = sqlx::query_as::<_, SupplierRow>(
        "SELECT id, branch_id, name, rif, phone, email, address, contact_person,
                is_self, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM supplier WHERE id = ?1 AND deleted_at IS NULL"
    )
    .bind(&req.supplier_id)
    .fetch_one(&state.db.pool)
    .await?;
    let supplier = supp.into_model();

    let delivery = SupplierDelivery::new(
        state.config.branch_id.clone(),
        supplier.id.clone(),
        supplier.name.clone(),
        req.transport_plate.unwrap_or_default(),
        req.transport_driver.unwrap_or_default(),
    );
    let mut delivery = delivery;
    delivery.notes = req.notes.unwrap_or_default();

    let mut total_cost = rust_decimal::Decimal::ZERO;
    let mut delivery_items = Vec::new();
    let mut fish_created = 0;

    for item_req in &req.items {
        validate_not_empty(&item_req.container_id, "contenedor")?;
        validate_not_empty(&item_req.fish_type_id, "tipo de pescado")?;
        validate_positive_i32(item_req.quantity, "cantidad")?;
        validate_weight(item_req.weight_grams)?;
        validate_non_negative_f64(item_req.unit_cost, "costo unitario")?;

        let container = sqlx::query_as::<_, ContainerLabelRow>(
            "SELECT label, fish_type_name FROM container WHERE id = ?1 AND deleted_at IS NULL"
        )
        .bind(&item_req.container_id)
        .fetch_one(&state.db.pool)
        .await?;

        let unit_cost = rust_decimal::Decimal::from_f64_retain(item_req.unit_cost)
            .ok_or_else(|| ApiError::bad_request("costo unitario inválido"))?;

        let di = SupplierDeliveryItem::new(
            delivery.id.clone(),
            item_req.container_id.clone(),
            container.label.clone(),
            item_req.fish_type_id.clone(),
            container.fish_type_name.clone(),
            item_req.quantity,
            item_req.weight_grams,
            unit_cost,
        );

        sqlx::query(
            "INSERT INTO supplier_delivery_item (id, delivery_id, container_id, container_label,
             fish_type_id, fish_type_name, quantity, weight_grams, unit_cost, op_counter, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        )
        .bind(&di.id)
        .bind(&di.delivery_id)
        .bind(&di.container_id)
        .bind(&di.container_label)
        .bind(&di.fish_type_id)
        .bind(&di.fish_type_name)
        .bind(di.quantity)
        .bind(di.weight_grams)
        .bind(di.unit_cost.to_string())
        .bind(di.op_counter)
        .bind(di.updated_at)
        .execute(&state.db.pool)
        .await?;

        let weight_per_fish = item_req.weight_grams / item_req.quantity;
        let mut created_fish: Vec<FishItem> = Vec::with_capacity(item_req.quantity as usize);
        for _ in 0..item_req.quantity {
            let fish = FishItem::new(
                state.config.branch_id.clone(),
                item_req.container_id.clone(),
                container.label.clone(),
                item_req.fish_type_id.clone(),
                container.fish_type_name.clone(),
                weight_per_fish,
            );
            let mut fish = fish;
            fish.supplier_delivery_item_id = Some(di.id.clone());
            fish.cost_price = Some(unit_cost);

            sqlx::query(
                "INSERT INTO fish_item (id, branch_id, container_id, container_label, fish_type_id,
                 fish_type_name, weight_grams, added_at, supplier_delivery_item_id, cost_price,
                 op_counter, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
            )
            .bind(&fish.id)
            .bind(&fish.branch_id)
            .bind(&fish.container_id)
            .bind(&fish.container_label)
            .bind(&fish.fish_type_id)
            .bind(&fish.fish_type_name)
            .bind(fish.weight_grams)
            .bind(fish.added_at)
            .bind(&fish.supplier_delivery_item_id)
            .bind(fish.cost_price.map(|c| c.to_string()))
            .bind(fish.op_counter)
            .bind(fish.updated_at)
            .execute(&state.db.pool)
            .await?;

            created_fish.push(fish);
        }
        crate::sync::push_sync_batch(&state, "FishItem", &created_fish).await;
        fish_created += created_fish.len() as i32;

        sqlx::query("UPDATE container SET current_count = current_count + ?1 WHERE id = ?2")
            .bind(item_req.quantity)
            .bind(&item_req.container_id)
            .execute(&state.db.pool)
            .await?;

        let item_cost = unit_cost * rust_decimal::Decimal::from(item_req.weight_grams) / rust_decimal::Decimal::from(1000);
        total_cost += item_cost;
        delivery_items.push(di);
    }

    delivery.total_cost = total_cost;

    sqlx::query(
        "INSERT INTO supplier_delivery (id, branch_id, supplier_id, supplier_name, delivery_date,
         notes, transport_plate, transport_driver, total_cost, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    )
    .bind(&delivery.id)
    .bind(&delivery.branch_id)
    .bind(&delivery.supplier_id)
    .bind(&delivery.supplier_name)
    .bind(delivery.delivery_date)
    .bind(&delivery.notes)
    .bind(&delivery.transport_plate)
    .bind(&delivery.transport_driver)
    .bind(delivery.total_cost.to_string())
    .bind(delivery.op_counter)
    .bind(delivery.updated_at)
    .execute(&state.db.pool)
    .await?;

    crate::sync::push_sync(&state, "SupplierDelivery", &delivery).await;
    for item in &delivery_items {
        crate::sync::push_sync(&state, "SupplierDeliveryItem", item).await;
    }

    Ok(Json(DeliveryResponse {
        delivery,
        items: delivery_items,
        fish_created,
    }))
}

pub async fn list_deliveries(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Vec<SupplierDelivery>> {
    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);
    let rows = sqlx::query_as::<_, SupplierDeliveryRow>(
        "SELECT id, branch_id, supplier_id, supplier_name, delivery_date, notes,
                transport_plate, transport_driver, total_cost,
                op_counter, updated_at, synced_at, deleted_at
         FROM supplier_delivery WHERE deleted_at IS NULL
         ORDER BY delivery_date DESC LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

pub async fn get_delivery(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<serde_json::Value> {
    let delivery = sqlx::query_as::<_, SupplierDeliveryRow>(
        "SELECT id, branch_id, supplier_id, supplier_name, delivery_date, notes,
                transport_plate, transport_driver, total_cost,
                op_counter, updated_at, synced_at, deleted_at
         FROM supplier_delivery WHERE id = ?1 AND deleted_at IS NULL"
    )
    .bind(&id)
    .fetch_one(&state.db.pool)
    .await?;

    let items = sqlx::query_as::<_, SupplierDeliveryItemRow>(
        "SELECT id, delivery_id, container_id, container_label, fish_type_id, fish_type_name,
                quantity, weight_grams, unit_cost, op_counter, updated_at, synced_at, deleted_at
         FROM supplier_delivery_item WHERE delivery_id = ?1 AND deleted_at IS NULL"
    )
    .bind(&id)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(serde_json::json!({
        "delivery": delivery.into_model(),
        "items": items.into_iter().map(|i| i.into_model()).collect::<Vec<SupplierDeliveryItem>>(),
    })))
}

// Row types
#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct SupplierRow {
    id: String,
    branch_id: String,
    name: String,
    rif: Option<String>,
    phone: String,
    email: Option<String>,
    address: Option<String>,
    contact_person: String,
    is_self: bool,
    is_active: bool,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

impl SupplierRow {
    fn into_model(self) -> Supplier {
        Supplier {
            id: self.id,
            branch_id: self.branch_id,
            name: self.name,
            rif: self.rif,
            phone: self.phone,
            email: self.email,
            address: self.address,
            contact_person: self.contact_person,
            is_self: self.is_self,
            is_active: self.is_active,
            op_counter: self.op_counter,
            updated_at: self.updated_at.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse updated_at '{}': {}", self.updated_at, e);
                Default::default()
            }),
            synced_at: self.synced_at.and_then(|s| s.parse().ok()),
            deleted_at: self.deleted_at.and_then(|s| s.parse().ok()),
        }
    }
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct SupplierDeliveryRow {
    id: String,
    branch_id: String,
    supplier_id: String,
    supplier_name: String,
    delivery_date: String,
    notes: String,
    transport_plate: String,
    transport_driver: String,
    total_cost: String,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

impl SupplierDeliveryRow {
    fn into_model(self) -> SupplierDelivery {
        SupplierDelivery {
            id: self.id,
            branch_id: self.branch_id,
            supplier_id: self.supplier_id,
            supplier_name: self.supplier_name,
            delivery_date: self.delivery_date.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse delivery_date '{}': {}", self.delivery_date, e);
                Default::default()
            }),
            notes: self.notes,
            transport_plate: self.transport_plate,
            transport_driver: self.transport_driver,
            total_cost: self.total_cost.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse total_cost '{}': {}", self.total_cost, e);
                Default::default()
            }),
            op_counter: self.op_counter,
            updated_at: self.updated_at.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse updated_at '{}': {}", self.updated_at, e);
                Default::default()
            }),
            synced_at: self.synced_at.and_then(|s| s.parse().ok()),
            deleted_at: self.deleted_at.and_then(|s| s.parse().ok()),
        }
    }
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct SupplierDeliveryItemRow {
    id: String,
    delivery_id: String,
    container_id: String,
    container_label: String,
    fish_type_id: String,
    fish_type_name: String,
    quantity: i32,
    weight_grams: i32,
    unit_cost: String,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

impl SupplierDeliveryItemRow {
    fn into_model(self) -> SupplierDeliveryItem {
        SupplierDeliveryItem {
            id: self.id,
            delivery_id: self.delivery_id,
            container_id: self.container_id,
            container_label: self.container_label,
            fish_type_id: self.fish_type_id,
            fish_type_name: self.fish_type_name,
            quantity: self.quantity,
            weight_grams: self.weight_grams,
            unit_cost: self.unit_cost.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse unit_cost '{}': {}", self.unit_cost, e);
                Default::default()
            }),
            op_counter: self.op_counter,
            updated_at: self.updated_at.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse updated_at '{}': {}", self.updated_at, e);
                Default::default()
            }),
            synced_at: self.synced_at.and_then(|s| s.parse().ok()),
            deleted_at: self.deleted_at.and_then(|s| s.parse().ok()),
        }
    }
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct ContainerLabelRow {
    label: String,
    fish_type_name: String,
}
