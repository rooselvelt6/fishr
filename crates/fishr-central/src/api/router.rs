use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::{HeaderName, HeaderValue, header::CONTENT_TYPE};
use crate::AppState;
use super::{sync, dashboard};

static SYNC_LIMITER: OnceLock<Mutex<HashMap<String, Vec<Instant>>>> = OnceLock::new();

pub async fn sync_rate_limit(
    request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    if ip != "unknown" {
        let map = SYNC_LIMITER.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = map.lock().unwrap();
        let timestamps = map.entry(ip.to_string()).or_default();
        let now = Instant::now();
        timestamps.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        if timestamps.len() >= 100 {
            return Err(axum::http::StatusCode::TOO_MANY_REQUESTS);
        }
        timestamps.push(now);
    }
    Ok(next.run(request).await)
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:8080".parse::<HeaderValue>().unwrap(),
            "http://localhost:9090".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:8080".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:9090".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers([CONTENT_TYPE]);

    let health = Router::new()
        .route("/api/health", get(|| async { "🐟 Central OK" }));

    let sync_route = Router::new()
        .route("/api/sync/push", post(sync::handle_sync_push))
        .layer(middleware::from_fn(sync_rate_limit));

    let dashboard_routes = Router::new()
        .route("/api/dashboard/overview", get(dashboard::overview))
        .route("/api/dashboard/branches", get(dashboard::list_branches))
        .route("/api/dashboard/branch/{id}/sales", get(dashboard::branch_sales));

    health.merge(sync_route).merge(dashboard_routes).with_state(state)
        .layer(cors)
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
}
