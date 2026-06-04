use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::http::StatusCode;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;

use crate::state::AppState;

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

fn hash_token(token: &str) -> String {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(token.as_bytes());
    format!("{:x}", hash)
}

pub async fn auth_required(
    State(state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = match request
        .headers()
        .get("x-session-token")
        .and_then(|v| v.to_str().ok())
    {
        Some(t) if !t.is_empty() => t,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let token_hash = hash_token(token);
    let now = chrono::Utc::now().to_rfc3339();

    let valid = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM user_session s
         JOIN user_account u ON s.user_id = u.id
         WHERE s.token_hash = ?1 AND s.expires_at > ?2 AND u.is_active = 1"
    )
    .bind(&token_hash)
    .bind(&now)
    .fetch_optional(&state.db.pool)
    .await
    .map(|r| r.is_some())
    .unwrap_or(false);

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
