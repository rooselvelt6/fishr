use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::http::StatusCode;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;

pub struct SlidingWindowCounter {
    max_requests: usize,
    window_secs: u64,
    requests: Mutex<HashMap<String, Vec<Instant>>>,
}

impl SlidingWindowCounter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self { max_requests, window_secs, requests: Mutex::new(HashMap::new()) }
    }

    pub fn try_consume(&self, key: &str) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        let mut map = self.requests.lock().unwrap();
        let timestamps = map.entry(key.to_string()).or_default();
        timestamps.retain(|t| now.duration_since(*t) < window);
        if timestamps.len() >= self.max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

pub static LOGIN_RATE_LIMIT: Lazy<SlidingWindowCounter> =
    Lazy::new(|| SlidingWindowCounter::new(10, 60));

pub fn client_ip<B>(request: &Request<B>) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn login_rate_limit(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = client_ip(&request);
    if !LOGIN_RATE_LIMIT.try_consume(&ip) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    Ok(next.run(request).await)
}

pub async fn auth_required(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let has_token = request
        .headers()
        .get("x-session-token")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| !s.is_empty());

    if !has_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
