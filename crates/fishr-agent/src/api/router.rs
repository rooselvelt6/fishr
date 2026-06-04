use axum::Router;
use axum::middleware;
use axum::routing::{get, post, put, delete};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::{HeaderName, HeaderValue, header::{CONTENT_TYPE, AUTHORIZATION}};

use crate::state::AppState;
use crate::api::analytics;
use crate::api::auth;
use crate::api::inventory;
use crate::api::pricing;
use crate::api::planner;
use crate::api::pos;
use crate::api::customers;
use crate::api::rate_limit;
use crate::api::reports;
use crate::api::setup;
use crate::api::suppliers;

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            "http://localhost:8080".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:8080".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, "x-session-token".parse().unwrap()])
        .expose_headers([CONTENT_TYPE, "x-session-token".parse().unwrap()])
}

async fn root() -> impl axum::response::IntoResponse {
    axum::response::Html(include_str!("../frontend/index.html"))
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let health = Router::new()
        .route("/api/health", get(|| async { "🐟 OK" }))
        .with_state(state.clone());

    let login = Router::new()
        .route("/api/auth/login", post(auth::login))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit::login_rate_limit))
        .with_state(state.clone());

    let frontend = Router::new()
        .route("/", get(root));

    let protected = Router::new()
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/setup", post(setup::setup_branch))
        .route("/api/fish-types", get(inventory::list_fish_types))
        .route("/api/fish-types", post(inventory::create_fish_type))
        .route("/api/fish-types/{id}", put(inventory::update_fish_type))
        .route("/api/containers", get(inventory::list_containers))
        .route("/api/containers", post(inventory::create_container))
        .route("/api/containers/{id}", put(inventory::update_container))
        .route("/api/containers/{id}", delete(inventory::delete_container))
        .route("/api/fish", post(inventory::add_fish))
        .route("/api/fish/available", get(inventory::list_available_fish))
        .route("/api/fish/{id}", delete(inventory::remove_fish))
        .route("/api/market-prices", get(inventory::list_market_prices))
        .route("/api/market-prices", post(inventory::set_market_price))
        .route("/api/pricing/suggested", get(pricing::suggested_prices))
        .route("/api/pos/calculate", post(pos::calculate_sale))
        .route("/api/pos/suggestions", post(pos::sale_suggestions))
        .route("/api/pos/confirm", post(pos::confirm_sale))
        .route("/api/pos/payment-methods", get(pos::list_payment_methods))
        .route("/api/preparations", get(pos::list_preparations))
        .route("/api/customers", get(customers::list_customers))
        .route("/api/customers", post(customers::create_customer))
        .route("/api/customers/{id}", get(customers::get_customer))
        .route("/api/customers/{id}", put(customers::update_customer))
        .route("/api/customers/search", get(customers::search_customers))
        .route("/api/sales", get(reports::list_sales))
        .route("/api/sales/{id}", get(reports::get_sale))
        .route("/api/reports/daily", get(reports::daily_report))
        .route("/api/reports/inventory-value", get(reports::inventory_valuation))
        .route("/api/scale/read", get(crate::hardware::scale::api_read_weight))
        .route("/api/scale/tare", post(crate::hardware::scale::api_tare))
        .route("/api/suppliers", get(suppliers::list_suppliers))
        .route("/api/suppliers", post(suppliers::create_supplier))
        .route("/api/suppliers/{id}", get(suppliers::get_supplier))
        .route("/api/suppliers/{id}", put(suppliers::update_supplier))
        .route("/api/suppliers/{id}", delete(suppliers::delete_supplier))
        .route("/api/suppliers/deliveries", get(suppliers::list_deliveries))
        .route("/api/suppliers/deliveries", post(suppliers::create_delivery))
        .route("/api/suppliers/deliveries/{id}", get(suppliers::get_delivery))
        .route("/api/sync/status", get(crate::sync::agent::api_sync_status))
        .route("/api/sync/trigger", post(crate::sync::agent::api_trigger_sync))
        .route("/api/print/receipt", post(crate::hardware::printer::api_print_receipt))
        .route("/api/analytics/dashboard", get(analytics::dashboard))
        .route("/api/analytics/sales-trend", get(analytics::sales_trend))
        .route("/api/analytics/top-products", get(analytics::top_products))
        .route("/api/planner/suggestions", get(planner::plan_inventory))
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit::auth_required))
        .with_state(state.clone());

    let app = health.merge(login).merge(protected).merge(frontend);

    app.layer(cors_layer())
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        ))
}
