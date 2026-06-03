#[cfg(feature = "hardware")]
pub mod scale;
#[cfg(feature = "hardware")]
pub mod printer;

#[cfg(not(feature = "hardware"))]
#[allow(dead_code)]
pub mod scale {
    use axum::Json;

    pub fn init_scale(_port: &str, _baud: u32) {
        tracing::info!("Hardware feature disabled, báscula no disponible");
    }

    pub fn read_weight() -> Result<f64, String> {
        Err("Hardware feature disabled".into())
    }

    pub fn tare() -> Result<(), String> {
        Err("Hardware feature disabled".into())
    }

    pub async fn api_read_weight() -> Json<serde_json::Value> {
        Json(serde_json::json!({ "weight_grams": 0.0, "success": false, "error": "hardware disabled" }))
    }

    pub async fn api_tare() -> Json<serde_json::Value> {
        Json(serde_json::json!({ "success": false, "error": "hardware disabled" }))
    }
}

#[cfg(not(feature = "hardware"))]
#[allow(dead_code)]
pub mod printer {
    use axum::Json;
    use serde_json::Value;

    pub fn init_printer(_port: &str, _baud: u32) {
        tracing::info!("Hardware feature disabled, impresora no disponible");
    }

    pub fn print_receipt(_data: &Value) -> Result<(), String> {
        Err("Hardware feature disabled".into())
    }

    pub async fn api_print_receipt(
        Json(_data): Json<Value>,
    ) -> Json<serde_json::Value> {
        Json(serde_json::json!({ "success": false, "error": "hardware disabled" }))
    }
}
