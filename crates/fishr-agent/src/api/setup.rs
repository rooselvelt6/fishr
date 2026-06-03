use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::api::error::{ApiResult, validate_not_empty};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SetupRequest {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub rif: String,
}

#[derive(Serialize)]
pub struct SetupResponse {
    pub branch_id: String,
    pub message: String,
}

pub async fn setup_branch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetupRequest>,
) -> ApiResult<SetupResponse> {
    validate_not_empty(&req.name, "nombre")?;
    validate_not_empty(&req.phone, "teléfono")?;
    validate_not_empty(&req.rif, "RIF")?;

    let branch_id = ulid::Ulid::new().to_string();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO branch (id, name, address, phone, rif, is_active, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)"
    )
    .bind(&branch_id)
    .bind(&req.name)
    .bind(&req.address)
    .bind(&req.phone)
    .bind(&req.rif)
    .bind(now.timestamp_millis())
    .bind(now)
    .execute(&state.db.pool)
    .await?;

    Ok(Json(SetupResponse {
        branch_id,
        message: "Sucursal configurada exitosamente".into(),
    }))
}
