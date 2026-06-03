use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug)]
pub struct Scale {
    port: Option<Box<dyn serialport::SerialPort>>,
    protocol: ScaleProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleProtocol {
    Continuous,
    Command { command: Vec<u8>, response_size: usize },
}

impl Scale {
    pub fn new(port_name: &str, baud_rate: u32, protocol: ScaleProtocol) -> Result<Self, String> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(1000))
            .open()
            .map_err(|e| format!("Error abriendo puerto {}: {}", port_name, e));

        match port {
            Ok(p) => Ok(Self { port: Some(p), protocol }),
            Err(e) => {
                tracing::warn!("Báscula no disponible: {}", e);
                Ok(Self { port: None, protocol })
            }
        }
    }

    pub fn read_weight(&mut self) -> Result<f64, String> {
        let port = self.port.as_mut().ok_or("Báscula no conectada")?;

        match &self.protocol {
            ScaleProtocol::Continuous => {
                let mut buf = [0u8; 32];
                port.read(&mut buf).map_err(|e| format!("Error lectura: {}", e))?;
                parse_weight(&buf)
            }
            ScaleProtocol::Command { command, response_size } => {
                port.write(command).map_err(|e| format!("Error escritura: {}", e))?;
                let mut buf = vec![0u8; *response_size];
                port.read(&mut buf).map_err(|e| format!("Error lectura: {}", e))?;
                parse_weight(&buf)
            }
        }
    }

    pub fn tare(&mut self) -> Result<(), String> {
        let port = self.port.as_mut().ok_or("Báscula no conectada")?;
        port.write(b"T\r\n").map_err(|e| format!("Error tara: {}", e))?;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }
}

fn parse_weight(data: &[u8]) -> Result<f64, String> {
    let s = String::from_utf8_lossy(data).trim().to_string();
    // Try to extract number from common formats
    for token in s.split_whitespace() {
        if let Ok(w) = token.parse::<f64>() {
            return Ok(w);
        }
    }
    Err(format!("No se pudo interpretar peso: '{}'", s))
}

// Singleton for the scale
use once_cell::sync::Lazy;
static SCALE: Lazy<Mutex<Option<Scale>>> = Lazy::new(|| Mutex::new(None));

pub fn init_scale(port_name: &str, baud_rate: u32) {
    let scale = Scale::new(port_name, baud_rate, ScaleProtocol::Continuous).ok();
    let mut s = SCALE.lock().unwrap();
    *s = scale;
}

pub fn read_weight() -> Result<f64, String> {
    let mut s = SCALE.lock().unwrap();
    match s.as_mut() {
        Some(scale) => scale.read_weight(),
        None => Err("Báscula no inicializada".into()),
    }
}

pub fn tare() -> Result<(), String> {
    let mut s = SCALE.lock().unwrap();
    match s.as_mut() {
        Some(scale) => scale.tare(),
        None => Err("Báscula no inicializada".into()),
    }
}

// API handlers for scale
use axum::{Json, extract::State};
use std::sync::Arc;
use crate::state::AppState;

pub async fn api_read_weight() -> Json<serde_json::Value> {
    match read_weight() {
        Ok(weight) => Json(serde_json::json!({ "weight_grams": weight, "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e, "success": false })),
    }
}

pub async fn api_tare() -> Json<serde_json::Value> {
    match tare() {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e, "success": false })),
    }
}
