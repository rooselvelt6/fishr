use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zeroize::Zeroize;
use crate::api::error::{ApiError, ApiResult};
use crate::state::AppState;

// --- Extractor for authenticated users ---

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("x-session-token")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Se requiere autenticación"})),
                )
            })?;

        let token_hash = hash_token(token);
        let now = chrono::Utc::now().to_rfc3339();

        let user = sqlx::query_as::<_, SessionUserRow>(
            "SELECT u.id, u.username, u.display_name, u.role
             FROM user_session s
             JOIN user_account u ON s.user_id = u.id
             WHERE s.token_hash = ?1 AND s.expires_at > ?2 AND u.is_active = 1"
        )
        .bind(&token_hash)
        .bind(&now)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Error de base de datos: {}", e)})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Sesión inválida o expirada"})),
            )
        })?;

        Ok(AuthUser {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
        })
    }
}

// --- Request/Response types ---

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub username: String,
    pub display_name: String,
    pub role: String,
}

impl From<AuthUser> for UserInfo {
    fn from(u: AuthUser) -> Self {
        UserInfo {
            username: u.username,
            display_name: u.display_name,
            role: u.role,
        }
    }
}

// --- Handlers ---

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(mut req): Json<LoginRequest>,
) -> ApiResult<LoginResponse> {
    if req.username.trim().is_empty() || req.password.is_empty() {
        return Err(ApiError::bad_request("Usuario y contraseña requeridos"));
    }

    let user = sqlx::query_as::<_, UserAccountRow>(
        "SELECT id, username, password_hash, display_name, role
         FROM user_account WHERE username = ?1 AND is_active = 1"
    )
    .bind(&req.username)
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| ApiError::unauthorized("Usuario o contraseña incorrectos"))?;

    let verified = verify_password(&req.password, &user.password_hash);
    req.password.zeroize();
    if !verified {
        return Err(ApiError::unauthorized("Usuario o contraseña incorrectos"));
    }

    let token = generate_token();
    let token_hash = hash_token(&token);
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::hours(12);
    let session_id = ulid::Ulid::new().to_string();

    sqlx::query(
        "INSERT INTO user_session (id, user_id, token_hash, expires_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )
    .bind(&session_id)
    .bind(&user.id)
    .bind(&token_hash)
    .bind(expires.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&state.db.pool)
    .await?;

    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            username: user.username,
            display_name: user.display_name,
            role: user.role,
        },
    }))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> ApiResult<&'static str> {
    sqlx::query("DELETE FROM user_session WHERE user_id = ?1")
        .bind(&auth.id)
        .execute(&state.db.pool)
        .await?;

    Ok(Json("Sesión cerrada"))
}

pub async fn me(auth: AuthUser) -> Json<UserInfo> {
    Json(auth.into())
}

// --- Helpers ---

fn hash_token(token: &str) -> String {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(token.as_bytes());
    format!("{:x}", hash)
}

fn generate_token() -> String {
    use rand::Rng;
    let mut chars: Vec<char> = rand::rngs::OsRng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();
    let token: String = chars.iter().collect();
    chars.zeroize();
    token
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    use argon2::Argon2;
    let parsed = match PasswordHash::new(hash) {
        Ok(p) => p,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

// --- Row types ---

#[derive(sqlx::FromRow)]
struct UserAccountRow {
    id: String,
    username: String,
    password_hash: String,
    display_name: String,
    role: String,
}

#[derive(sqlx::FromRow)]
struct SessionUserRow {
    id: String,
    username: String,
    display_name: String,
    role: String,
}
