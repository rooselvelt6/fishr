use serde_json::Value;
use std::sync::Mutex;

pub struct Printer {
    port: Option<Box<dyn serialport::SerialPort>>,
}

impl Printer {
    pub fn new(port_name: &str, baud_rate: u32) -> Self {
        let port = serialport::new(port_name, baud_rate)
            .timeout(std::time::Duration::from_millis(2000))
            .open()
            .ok();

        if port.is_none() {
            tracing::warn!("Impresora no disponible en {}", port_name);
        }

        Self { port }
    }

    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }

    pub fn print_receipt(&mut self, data: &Value) -> Result<(), String> {
        let port = self.port.as_mut().ok_or("Impresora no conectada")?;

        let branch = data["branch_name"].as_str().unwrap_or("PESCADERÍA");
        let rif = data["branch_rif"].as_str().unwrap_or("J-00000000-0");
        let address = data["branch_address"].as_str().unwrap_or("");
        let control = data["invoice_control"].as_str().unwrap_or("F-0001-00000001");
        let date = data["created_at"].as_str().unwrap_or("");
        let items = data["items"].as_array().map(|v| v.as_slice()).unwrap_or(&[]);
        let total = data["total"].as_str().unwrap_or("0.00");

        let mut buf = Vec::new();

        // ESC/POS commands
        buf.extend(b"\x1B\x40"); // Initialize printer
        buf.extend(b"\x1B\x61\x01"); // Center align

        // Header
        buf.extend(branch.as_bytes());
        buf.extend(b"\n");
        buf.extend(b"RIF: ");
        buf.extend(rif.as_bytes());
        buf.extend(b"\n");
        if !address.is_empty() {
            buf.extend(address.as_bytes());
            buf.extend(b"\n");
        }
        buf.extend(b"Factura: ");
        buf.extend(control.as_bytes());
        buf.extend(b"\n");
        buf.extend(date.as_bytes());
        buf.extend(b"\n");

        buf.extend(b"\x1B\x61\x00"); // Left align
        buf.extend(b"--------------------------------\n");

        // Items
        for item in items {
            let name = item["fish_type_name"].as_str().unwrap_or("Pescado");
            let weight = item["weight_grams"].as_f64().unwrap_or(0.0);
            let price = item["price_per_kg"].as_str().unwrap_or("0.00");
            let subtotal = item["subtotal"].as_str().unwrap_or("0.00");

            buf.extend(format!("{:<15}\n", name).as_bytes());
            buf.extend(format!("  {:>4.0}g x {} = {:>8}\n", weight, price, subtotal).as_bytes());

            if let Some(prep) = item["preparation_name"].as_str() {
                if !prep.is_empty() {
                    buf.extend(format!("  + {}: {}\n", prep, item["preparation_fee"].as_str().unwrap_or("0.00")).as_bytes());
                }
            }
        }

        buf.extend(b"--------------------------------\n");
        buf.extend(b"\x1B\x61\x01"); // Center

        // Totals
        buf.extend(format!("TOTAL: Bs. {}\n", total).as_bytes());
        buf.extend(b"\n");
        buf.extend(b"\x1B\x61\x00"); // Left
        buf.extend(b"Gracias por su visita!\n");
        buf.extend(b"\n\n\n");
        buf.extend(b"\x1B\x69"); // Cut paper

        port.write_all(&buf).map_err(|e| format!("Error impresion: {}", e))?;
        Ok(())
    }
}

use once_cell::sync::Lazy;
static PRINTER: Lazy<Mutex<Option<Printer>>> = Lazy::new(|| Mutex::new(None));

pub fn init_printer(port_name: &str, baud_rate: u32) {
    let printer = Printer::new(port_name, baud_rate);
    let mut p = PRINTER.lock().unwrap();
    *p = Some(printer);
}

pub fn print_receipt(data: &Value) -> Result<(), String> {
    let mut p = PRINTER.lock().unwrap();
    match p.as_mut() {
        Some(printer) => printer.print_receipt(data),
        None => Err("Impresora no inicializada".into()),
    }
}

// API handler
use axum::{Json, extract::State};
use std::sync::Arc;

pub async fn api_print_receipt(
    Json(data): Json<Value>,
) -> Json<serde_json::Value> {
    match print_receipt(&data) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e, "success": false })),
    }
}
