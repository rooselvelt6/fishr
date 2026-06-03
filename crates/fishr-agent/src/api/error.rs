use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};

pub type ApiResult<T> = Result<Json<T>, ApiError>;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message: msg.into() }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::NOT_FOUND, message: msg.into() }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: msg.into() }
    }

    #[allow(dead_code)]
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::CONFLICT, message: msg.into() }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self { status: StatusCode::UNAUTHORIZED, message: msg.into() }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({ "error": self.message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Self::not_found("Recurso no encontrado"),
            other => Self::internal(format!("Error de base de datos: {}", other)),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::internal(format!("Error de serialización: {}", e))
    }
}

// Input validators
pub fn validate_not_empty(val: &str, field: &str) -> Result<(), ApiError> {
    if val.trim().is_empty() {
        return Err(ApiError::bad_request(format!("{} no puede estar vacío", field)));
    }
    if val.len() > 200 {
        return Err(ApiError::bad_request(format!("{} es demasiado largo", field)));
    }
    Ok(())
}

pub fn validate_positive_i32(val: i32, field: &str) -> Result<(), ApiError> {
    if val <= 0 {
        return Err(ApiError::bad_request(format!("{} debe ser mayor a 0", field)));
    }
    if val > 100_000 {
        return Err(ApiError::bad_request(format!("{} es demasiado grande", field)));
    }
    Ok(())
}

pub fn validate_weight(val: i32) -> Result<(), ApiError> {
    if val <= 0 {
        return Err(ApiError::bad_request("El peso debe ser mayor a 0 gramos"));
    }
    if val > 100_000 {
        return Err(ApiError::bad_request("El peso no puede exceder 100 kg"));
    }
    Ok(())
}

pub fn validate_non_negative_f64(val: f64, field: &str) -> Result<(), ApiError> {
    if val.is_nan() || val.is_infinite() || val < 0.0 {
        return Err(ApiError::bad_request(format!("{} no es válido", field)));
    }
    if val > 1_000_000.0 {
        return Err(ApiError::bad_request(format!("{} es demasiado grande", field)));
    }
    Ok(())
}
