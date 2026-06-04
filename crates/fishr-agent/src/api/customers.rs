use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use fishr_core::models::*;
use crate::api::error::{ApiResult, validate_not_empty};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn list_customers(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Vec<Customer>> {
    let rows = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, branch_id, name, phone, email, rif, address, points,
                op_counter, updated_at, synced_at, deleted_at
         FROM customer WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub rif: Option<String>,
    pub address: Option<String>,
}

pub async fn create_customer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCustomerRequest>,
) -> ApiResult<Customer> {
    validate_not_empty(&req.name, "nombre")?;
    validate_not_empty(&req.phone, "teléfono")?;

    let customer = Customer::new(state.config.branch_id.clone(), req.name, req.phone);
    let mut c = customer.clone();
    c.email = req.email;
    c.rif = req.rif;
    c.address = req.address;

    sqlx::query(
        "INSERT INTO customer (id, branch_id, name, phone, email, rif, address, points, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)"
    )
    .bind(&c.id)
    .bind(&c.branch_id)
    .bind(&c.name)
    .bind(&c.phone)
    .bind(&c.email)
    .bind(&c.rif)
    .bind(&c.address)
    .bind(c.op_counter)
    .bind(c.updated_at)
    .execute(&state.db.pool)
    .await?;

    crate::api::inventory::push_sync(&state, "Customer", &c).await;
    Ok(Json(c))
}

pub async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Option<Customer>> {
    let row = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, branch_id, name, phone, email, rif, address, points,
                op_counter, updated_at, synced_at, deleted_at
         FROM customer WHERE id = ?1 AND deleted_at IS NULL"
    )
    .bind(&id)
    .fetch_optional(&state.db.pool)
    .await?;

    Ok(Json(row.map(|r| r.into_model())))
}

pub async fn update_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<CreateCustomerRequest>,
) -> ApiResult<Customer> {
    validate_not_empty(&req.name, "nombre")?;
    validate_not_empty(&req.phone, "teléfono")?;

    let now = chrono::Utc::now();
    sqlx::query(
        "UPDATE customer SET name=?1, phone=?2, email=?3, rif=?4, address=?5, updated_at=?6, op_counter=?7 WHERE id=?8"
    )
    .bind(&req.name)
    .bind(&req.phone)
    .bind(&req.email)
    .bind(&req.rif)
    .bind(&req.address)
    .bind(now)
    .bind(now.timestamp_millis())
    .bind(&id)
    .execute(&state.db.pool)
    .await?;

    let row = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, branch_id, name, phone, email, rif, address, points,
                op_counter, updated_at, synced_at, deleted_at
         FROM customer WHERE id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db.pool)
    .await?;

    let c = row.into_model();
    crate::api::inventory::push_sync(&state, "Customer", &c).await;
    Ok(Json(c))
}

pub async fn search_customers(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Vec<Customer>> {
    let pattern = format!("%{}%", q.q);
    let rows = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, branch_id, name, phone, email, rif, address, points,
                op_counter, updated_at, synced_at, deleted_at
         FROM customer WHERE deleted_at IS NULL AND (name LIKE ?1 OR phone LIKE ?1 OR rif LIKE ?1)
         ORDER BY name LIMIT 20"
    )
    .bind(&pattern)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(Json(rows.into_iter().map(|r| r.into_model()).collect()))
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct CustomerRow {
    id: String,
    branch_id: String,
    name: String,
    phone: String,
    email: Option<String>,
    rif: Option<String>,
    address: Option<String>,
    points: i64,
    op_counter: i64,
    updated_at: String,
    synced_at: Option<String>,
    deleted_at: Option<String>,
}

impl CustomerRow {
    fn into_model(self) -> Customer {
        Customer {
            id: self.id,
            branch_id: self.branch_id,
            name: self.name,
            phone: self.phone,
            email: self.email,
            rif: self.rif,
            address: self.address,
            points: self.points,
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
