use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use fishr_core::models::*;
use crate::api::error::{ApiResult, ApiError, validate_not_empty, validate_positive_i32, validate_weight, validate_non_negative_f64};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_fish_types(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Vec<FishType>> {
    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);
    let rows = sqlx::query_as::<_, FishTypeRow>(
        "SELECT id, name, species, category, description, op_counter, updated_at, synced_at, deleted_at
         FROM fish_type WHERE deleted_at IS NULL ORDER BY name LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

pub async fn list_containers(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Vec<Container>> {
    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);
    let rows = sqlx::query_as::<_, ContainerRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count,
                location, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM container WHERE deleted_at IS NULL ORDER BY label LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

#[derive(Deserialize)]
pub struct CreateContainerRequest {
    pub fish_type_id: String,
    pub label: String,
    pub capacity: i32,
    pub location: String,
}

pub async fn create_container(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateContainerRequest>,
) -> ApiResult<Container> {
    validate_not_empty(&req.label, "etiqueta")?;
    validate_positive_i32(req.capacity, "capacidad")?;
    validate_not_empty(&req.fish_type_id, "tipo de pescado")?;

    let branch_id = &state.config.branch_id;

    let fish_type = sqlx::query_as::<_, FishTypeRow>(
        "SELECT id, name, species, category, description, op_counter, updated_at, synced_at, deleted_at
         FROM fish_type WHERE id = ?1"
    )
    .bind(&req.fish_type_id)
    .fetch_one(&state.db.pool)
    .await?;

    let container = Container::new(
        branch_id.clone(),
        req.fish_type_id,
        fish_type.name,
        req.label,
        req.capacity,
        req.location,
    );

    sqlx::query(
        "INSERT INTO container (id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count, location, is_active, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, 1, ?8, ?9)"
    )
    .bind(&container.id)
    .bind(&container.branch_id)
    .bind(&container.fish_type_id)
    .bind(&container.fish_type_name)
    .bind(&container.label)
    .bind(container.capacity)
    .bind(&container.location)
    .bind(container.op_counter)
    .bind(container.updated_at)
    .execute(&state.db.pool)
    .await?;

    crate::sync::push_sync(&state, "Container", &container).await;
    Ok(Json(container))
}

pub async fn update_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CreateContainerRequest>,
) -> ApiResult<Container> {
    validate_not_empty(&req.label, "etiqueta")?;
    validate_positive_i32(req.capacity, "capacidad")?;
    validate_not_empty(&req.fish_type_id, "tipo de pescado")?;

    let now = chrono::Utc::now();

    let fish_type = sqlx::query_as::<_, FishTypeRow>(
        "SELECT id, name, species, category, description, op_counter, updated_at, synced_at, deleted_at
         FROM fish_type WHERE id = ?1"
    )
    .bind(&req.fish_type_id)
    .fetch_one(&state.db.pool)
    .await?;

    sqlx::query(
        "UPDATE container SET fish_type_id=?1, fish_type_name=?2, label=?3, capacity=?4, location=?5,
         updated_at=?6, op_counter=?7 WHERE id=?8"
    )
    .bind(&req.fish_type_id)
    .bind(&fish_type.name)
    .bind(&req.label)
    .bind(req.capacity)
    .bind(&req.location)
    .bind(now)
    .bind(now.timestamp_millis())
    .bind(&id)
    .execute(&state.db.pool)
    .await?;

    let row = sqlx::query_as::<_, ContainerRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count,
                location, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM container WHERE id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db.pool)
    .await?;

    let container = row.into_model();
    crate::sync::push_sync(&state, "Container", &container).await;
    Ok(Json(container))
}

pub async fn delete_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<&'static str> {
    let now = chrono::Utc::now();
    sqlx::query("UPDATE container SET deleted_at=?1, updated_at=?2, op_counter=?3 WHERE id=?4")
        .bind(now)
        .bind(now)
        .bind(now.timestamp_millis())
        .bind(&id)
        .execute(&state.db.pool)
        .await?;

    Ok(Json("Eliminado"))
}

#[derive(Deserialize)]
pub struct AddFishRequest {
    pub container_id: String,
    pub weight_grams: i32,
    pub count: i32,
    pub _supplier_delivery_item_id: Option<String>,
    pub cost_price: Option<f64>,
}

pub async fn add_fish(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddFishRequest>,
) -> ApiResult<Vec<FishItem>> {
    validate_not_empty(&req.container_id, "contenedor")?;
    validate_weight(req.weight_grams)?;
    validate_positive_i32(req.count, "cantidad")?;
    if let Some(cp) = req.cost_price {
        validate_non_negative_f64(cp, "costo")?;
    }

    let container = sqlx::query_as::<_, ContainerRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count,
                location, is_active, op_counter, updated_at, synced_at, deleted_at
         FROM container WHERE id = ?1"
    )
    .bind(&req.container_id)
    .fetch_one(&state.db.pool)
    .await?;

    let mut items = Vec::new();
    for _ in 0..req.count {
        let fish = FishItem::new(
            container.branch_id.clone(),
            container.id.clone(),
            container.label.clone(),
            container.fish_type_id.clone(),
            container.fish_type_name.clone(),
            req.weight_grams,
        );

        sqlx::query(
            "INSERT INTO fish_item (id, branch_id, container_id, container_label, fish_type_id, fish_type_name,
             weight_grams, added_at, supplier_delivery_item_id, cost_price, op_counter, updated_at)
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

        items.push(fish);
    }

    crate::sync::push_sync_batch(&state, "FishItem", &items).await;

    let new_count = container.current_count + req.count;
    sqlx::query("UPDATE container SET current_count=?1 WHERE id=?2")
        .bind(new_count)
        .bind(&container.id)
        .execute(&state.db.pool)
        .await?;

    Ok(Json(items))
}

pub async fn list_available_fish(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Vec<FishItem>> {
    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);
    let rows = sqlx::query_as::<_, FishItemRow>(
        "SELECT id, branch_id, container_id, container_label, fish_type_id, fish_type_name,
                weight_grams, added_at, sold_at, sold_in_sale_id,
                supplier_delivery_item_id, cost_price,
                op_counter, updated_at, synced_at, deleted_at
         FROM fish_item WHERE sold_at IS NULL AND deleted_at IS NULL
         ORDER BY fish_type_name, added_at LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

pub async fn remove_fish(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<&'static str> {
    let now = chrono::Utc::now();
    sqlx::query("UPDATE fish_item SET deleted_at=?1, updated_at=?2, op_counter=?3 WHERE id=?4")
        .bind(now)
        .bind(now)
        .bind(now.timestamp_millis())
        .bind(&id)
        .execute(&state.db.pool)
        .await?;

    Ok(Json("Eliminado"))
}

#[derive(Deserialize)]
pub struct CreateFishTypeRequest {
    pub name: String,
    pub species: String,
    pub category: String,
    pub description: String,
}

pub async fn create_fish_type(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateFishTypeRequest>,
) -> ApiResult<FishType> {
    validate_not_empty(&req.name, "nombre")?;
    validate_not_empty(&req.species, "especie")?;
    validate_not_empty(&req.category, "categoría")?;

    let category: FishCategory = serde_json::from_str(&format!("\"{}\"", &req.category))
        .unwrap_or_default();

    let fish_type = FishType::new(req.name, req.species, category, req.description);
    let cat_str = category_to_string(&fish_type.category);

    sqlx::query(
        "INSERT INTO fish_type (id, name, species, category, description, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )
    .bind(&fish_type.id)
    .bind(&fish_type.name)
    .bind(&fish_type.species)
    .bind(&cat_str)
    .bind(&fish_type.description)
    .bind(fish_type.op_counter)
    .bind(fish_type.updated_at)
    .execute(&state.db.pool)
    .await?;

    crate::sync::push_sync(&state, "FishType", &fish_type).await;
    Ok(Json(fish_type))
}

pub async fn update_fish_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CreateFishTypeRequest>,
) -> ApiResult<FishType> {
    validate_not_empty(&req.name, "nombre")?;
    validate_not_empty(&req.species, "especie")?;
    validate_not_empty(&req.category, "categoría")?;

    let now = chrono::Utc::now();

    let category: FishCategory = serde_json::from_str(&format!("\"{}\"", &req.category))
        .unwrap_or_default();

    let cat_clean = category_to_string(&category);

    sqlx::query(
        "UPDATE fish_type SET name=?1, species=?2, category=?3, description=?4,
         updated_at=?5, op_counter=?6 WHERE id=?7"
    )
    .bind(&req.name)
    .bind(&req.species)
    .bind(&cat_clean)
    .bind(&req.description)
    .bind(now)
    .bind(now.timestamp_millis())
    .bind(&id)
    .execute(&state.db.pool)
    .await?;

    let row = sqlx::query_as::<_, FishTypeRow>(
        "SELECT id, name, species, category, description, op_counter, updated_at, synced_at, deleted_at
         FROM fish_type WHERE id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db.pool)
    .await?;

    let fish_type = row.into_model();
    crate::sync::push_sync(&state, "FishType", &fish_type).await;
    Ok(Json(fish_type))
}

fn category_to_string(cat: &FishCategory) -> String {
    cat.as_str().to_string()
}

pub async fn list_market_prices(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Vec<MarketPrice>> {
    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);
    let rows = sqlx::query_as::<_, MarketPriceRow>(
        "SELECT id, branch_id, fish_type_id, fish_type_name, price_per_kg, cost_price,
                effective_from, effective_to, op_counter, updated_at, synced_at, deleted_at
         FROM market_price WHERE effective_to IS NULL AND deleted_at IS NULL
         ORDER BY fish_type_name LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

#[derive(Deserialize)]
pub struct SetMarketPriceRequest {
    pub fish_type_id: String,
    pub price_per_kg: f64,
    pub cost_price: f64,
}

pub async fn set_market_price(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetMarketPriceRequest>,
) -> ApiResult<MarketPrice> {
    validate_not_empty(&req.fish_type_id, "tipo de pescado")?;
    validate_non_negative_f64(req.price_per_kg, "precio por kg")?;
    validate_non_negative_f64(req.cost_price, "precio de costo")?;

    let now = chrono::Utc::now();

    let fish_type = sqlx::query_as::<_, FishTypeRow>(
        "SELECT id, name, species, category, description, op_counter, updated_at, synced_at, deleted_at
         FROM fish_type WHERE id = ?1"
    )
    .bind(&req.fish_type_id)
    .fetch_one(&state.db.pool)
    .await?;

    sqlx::query("UPDATE market_price SET effective_to=?1 WHERE fish_type_id=?2 AND effective_to IS NULL")
        .bind(now)
        .bind(&req.fish_type_id)
        .execute(&state.db.pool)
        .await?;

    let price = rust_decimal::Decimal::from_f64_retain(req.price_per_kg)
        .ok_or_else(|| ApiError::bad_request("precio por kg inválido"))?;
    let cost = rust_decimal::Decimal::from_f64_retain(req.cost_price)
        .ok_or_else(|| ApiError::bad_request("precio de costo inválido"))?;

    let mp = MarketPrice::new(
        state.config.branch_id.clone(),
        req.fish_type_id,
        fish_type.name,
        price,
        cost,
    );

    sqlx::query(
        "INSERT INTO market_price (id, branch_id, fish_type_id, fish_type_name, price_per_kg, cost_price,
         effective_from, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
    )
    .bind(&mp.id)
    .bind(&mp.branch_id)
    .bind(&mp.fish_type_id)
    .bind(&mp.fish_type_name)
    .bind(mp.price_per_kg.to_string())
    .bind(mp.cost_price.to_string())
    .bind(mp.effective_from)
    .bind(mp.op_counter)
    .bind(mp.updated_at)
    .execute(&state.db.pool)
    .await?;

    crate::sync::push_sync(&state, "MarketPrice", &mp).await;
    Ok(Json(mp))
}

// Row types
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

impl FishTypeRow {
    fn into_model(self) -> FishType {
        FishType {
            id: self.id,
            name: self.name,
            species: self.species,
            category: match self.category.as_str() {
                "White" => FishCategory::White,
                "Blue" => FishCategory::Blue,
                "Shellfish" => FishCategory::Shellfish,
                "Crustacean" => FishCategory::Crustacean,
                "Other" => FishCategory::Other,
                _ => {
                    tracing::warn!("unknown FishCategory '{}', defaulting to White", self.category);
                    FishCategory::White
                }
            },
            description: self.description,
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

#[derive(sqlx::FromRow)]
pub(crate) struct ContainerRow {
    pub(crate) id: String,
    pub(crate) branch_id: String,
    pub(crate) fish_type_id: String,
    pub(crate) fish_type_name: String,
    pub(crate) label: String,
    pub(crate) capacity: i32,
    pub(crate) current_count: i32,
    pub(crate) location: String,
    pub(crate) is_active: bool,
    pub(crate) op_counter: i64,
    pub(crate) updated_at: String,
    pub(crate) synced_at: Option<String>,
    pub(crate) deleted_at: Option<String>,
}

impl ContainerRow {
    fn into_model(self) -> Container {
        Container {
            id: self.id,
            branch_id: self.branch_id,
            fish_type_id: self.fish_type_id,
            fish_type_name: self.fish_type_name,
            label: self.label,
            capacity: self.capacity,
            current_count: self.current_count,
            location: self.location,
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
    supplier_delivery_item_id: Option<String>,
    cost_price: Option<String>,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

impl FishItemRow {
    fn into_model(self) -> FishItem {
        FishItem {
            id: self.id,
            branch_id: self.branch_id,
            container_id: self.container_id,
            container_label: self.container_label,
            fish_type_id: self.fish_type_id,
            fish_type_name: self.fish_type_name,
            weight_grams: self.weight_grams,
            added_at: self.added_at.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse added_at '{}': {}", self.added_at, e);
                Default::default()
            }),
            sold_at: self.sold_at.and_then(|s| s.parse().ok()),
            sold_in_sale_id: self.sold_in_sale_id,
            supplier_delivery_item_id: self.supplier_delivery_item_id,
            cost_price: self.cost_price.and_then(|s| s.parse().ok()),
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

#[derive(sqlx::FromRow)]
pub(crate) struct MarketPriceRow {
    pub(crate) id: String,
    pub(crate) branch_id: String,
    pub(crate) fish_type_id: String,
    pub(crate) fish_type_name: String,
    pub(crate) price_per_kg: f64,
    pub(crate) cost_price: f64,
    pub(crate) effective_from: String,
    pub(crate) effective_to: Option<String>,
    pub(crate) op_counter: i64,
    pub(crate) updated_at: String,
    pub(crate) synced_at: Option<String>,
    pub(crate) deleted_at: Option<String>,
}

impl MarketPriceRow {
    fn into_model(self) -> MarketPrice {
        MarketPrice {
            id: self.id,
            branch_id: self.branch_id,
            fish_type_id: self.fish_type_id,
            fish_type_name: self.fish_type_name,
            price_per_kg: rust_decimal::Decimal::from_f64_retain(self.price_per_kg).unwrap_or_else(|| {
                tracing::warn!("failed to convert price_per_kg {} to Decimal", self.price_per_kg);
                Default::default()
            }),
            cost_price: rust_decimal::Decimal::from_f64_retain(self.cost_price).unwrap_or_else(|| {
                tracing::warn!("failed to convert cost_price {} to Decimal", self.cost_price);
                Default::default()
            }),
            effective_from: self.effective_from.parse().unwrap_or_else(|e| {
                tracing::warn!("failed to parse effective_from '{}': {}", self.effective_from, e);
                Default::default()
            }),
            effective_to: self.effective_to.and_then(|s| s.parse().ok()),
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


