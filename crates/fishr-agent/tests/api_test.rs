use std::sync::Arc;
use axum::http::{Request, StatusCode};
use axum::body::Body;
use tower::ServiceExt;
use http_body_util::BodyExt;
use fishr_agent::api::router::build_router;
use fishr_agent::state::{AppState, AgentConfig};
use fishr_agent::sync::SyncConfig;
use fishr_agent::db::Database;

fn test_agent_config() -> AgentConfig {
    AgentConfig {
        branch_id: "test-branch-001".into(),
        branch_name: "Test Sucursal".into(),
        branch_rif: "J-12345678-9".into(),
        branch_address: "Test Address".into(),
        branch_phone: "04121234567".into(),
        scale_port: None,
        printer_port: None,
    }
}

fn test_sync_config() -> SyncConfig {
    SyncConfig {
        central_url: String::new(),
        branch_id: "test-branch-001".into(),
        sync_interval_secs: 9999,
        max_batch_size: 100,
        retry_delay_secs: 60,
        max_retries: 3,
    }
}

fn hash_password(password: &str) -> String {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use argon2::Argon2;
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .unwrap_or_else(|_| String::new())
}

async fn setup_test_app() -> (Arc<AppState>, Arc<Database>) {
    let db = Arc::new(Database::new("sqlite://:memory:").await.unwrap());
    db.run_migrations().await.unwrap();

    let branch_id = "test-branch-001";
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO branch (id, name, address, phone, rif, is_active, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
    )
    .bind(branch_id)
    .bind("Test Sucursal")
    .bind("Test Address")
    .bind("04121234567")
    .bind("J-12345678-9")
    .bind(now.timestamp_millis())
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    let password_hash = hash_password("testpass123");
    sqlx::query(
        "INSERT INTO user_account (id, branch_id, username, password_hash, display_name, role, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)",
    )
    .bind("test-user-001")
    .bind(branch_id)
    .bind("testuser")
    .bind(&password_hash)
    .bind("Test User")
    .bind("cajero")
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&db.pool)
    .await
    .unwrap();

    let methods = [
        ("Efectivo", "Pago en efectivo"),
        ("Punto de Venta", "Tarjeta débito/crédito"),
    ];
    for (name, desc) in &methods {
        let id = ulid::Ulid::new().to_string();
        sqlx::query(
            "INSERT INTO payment_method (id, branch_id, name, description, is_active, op_counter, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
        )
        .bind(&id)
        .bind(branch_id)
        .bind(name)
        .bind(desc)
        .bind(now.timestamp_millis())
        .bind(now)
        .execute(&db.pool)
        .await
        .unwrap();
    }

    let config = test_agent_config();
    let sync_config = test_sync_config();
    let state = Arc::new(AppState { db: db.clone(), config, sync_config });
    (state, db)
}

async fn read_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn login_and_get_token(app: &axum::Router) -> String {
    let body = serde_json::json!({"username": "testuser", "password": "testpass123"});
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    json["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_health_check() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let req = Request::builder().uri("/api/health").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_wrong_password() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let body = serde_json::json!({"username": "testuser", "password": "wrongpass"});
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_success() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let body = serde_json::json!({"username": "testuser", "password": "testpass123"});
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert!(json["token"].as_str().unwrap().len() > 40);
    assert_eq!(json["user"]["username"], "testuser");
}

#[tokio::test]
async fn test_me_without_token() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let req = Request::builder().uri("/api/auth/me").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_me_with_valid_token() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;
    let req = Request::builder()
        .uri("/api/auth/me")
        .header("x-session-token", &token)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["username"], "testuser");
}

#[tokio::test]
async fn test_logout_clears_session() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("x-session-token", &token)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/api/auth/me")
        .header("x-session-token", &token)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_requires_auth() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let req = Request::builder().uri("/api/containers").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_payment_methods() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;
    let req = Request::builder()
        .uri("/api/pos/payment-methods")
        .header("x-session-token", &token)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    let methods: Vec<serde_json::Value> = serde_json::from_value(json).unwrap();
    assert!(!methods.is_empty());
}

#[tokio::test]
async fn test_list_containers_empty() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;
    let req = Request::builder()
        .uri("/api/containers")
        .header("x-session-token", &token)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_and_list_containers() {
    let (state, db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;

    let now = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO fish_type (id, name, op_counter, updated_at) VALUES (?1, ?2, ?3, ?4)"
    )
    .bind("ft-cont-test")
    .bind("Mero")
    .bind(now.timestamp_millis())
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    let body = serde_json::json!({
        "fish_type_id": "ft-cont-test",
        "label": "Container A",
        "capacity": 50,
        "location": "Tanque 1",
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/containers")
        .header("Content-Type", "application/json")
        .header("x-session-token", &token)
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/api/containers")
        .header("x-session-token", &token)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_customer() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;

    let body = serde_json::json!({
        "name": "Juan Pérez",
        "phone": "04121234567",
        "email": "juan@example.com",
        "rif": "V-12345678-9",
        "address": "Calle 1",
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/customers")
        .header("Content-Type", "application/json")
        .header("x-session-token", &token)
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_customer_validation() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;

    let body = serde_json::json!({"name": "", "phone": "", "email": "", "address": ""});
    let req = Request::builder()
        .method("POST")
        .uri("/api/customers")
        .header("Content-Type", "application/json")
        .header("x-session-token", &token)
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_pos_confirm_sale() {
    let (state, db) = setup_test_app().await;
    let app = build_router(state);
    let token = login_and_get_token(&app).await;

    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO fish_type (id, name, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind("ft-001")
    .bind("Mero")
    .bind(now.timestamp_millis())
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO container (id, branch_id, fish_type_id, fish_type_name, label, capacity, current_count, is_active, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9)",
    )
    .bind("cont-001")
    .bind("test-branch-001")
    .bind("ft-001")
    .bind("Mero")
    .bind("Mero Container")
    .bind(50)
    .bind(10)
    .bind(now.timestamp_millis())
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO market_price (id, branch_id, fish_type_id, fish_type_name, price_per_kg, effective_from, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind("mp-001")
    .bind("test-branch-001")
    .bind("ft-001")
    .bind("Mero")
    .bind("15.50")
    .bind(now)
    .bind(now.timestamp_millis())
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO fish_item (id, branch_id, container_id, container_label, fish_type_id, fish_type_name, weight_grams, added_at, op_counter, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind("fi-001")
    .bind("test-branch-001")
    .bind("cont-001")
    .bind("Mero Container")
    .bind("ft-001")
    .bind("Mero")
    .bind(1500i32)
    .bind(now)
    .bind(now.timestamp_millis())
    .bind(now)
    .execute(&db.pool)
    .await
    .unwrap();

    // Get the first payment method id
    let pm_id: String = sqlx::query_scalar("SELECT id FROM payment_method LIMIT 1")
        .fetch_one(&db.pool)
        .await
        .unwrap();

    let body = serde_json::json!({
        "items": [{
            "fish_item_id": "fi-001",
            "container_id": "cont-001",
            "weight_grams": 1500,
            "price_per_kg": 15.50,
            "preparation_id": null,
            "preparation_name": null,
            "preparation_fee": 0.0,
        }],
        "payment_method_id": pm_id,
        "customer_id": null,
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/pos/confirm")
        .header("Content-Type", "application/json")
        .header("x-session-token", &token)
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert!(json["sale_id"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn test_security_headers_present() {
    let (state, _db) = setup_test_app().await;
    let app = build_router(state);
    let req = Request::builder().uri("/api/health").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.headers().get("x-content-type-options").and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    assert_eq!(
        resp.headers().get("x-frame-options").and_then(|v| v.to_str().ok()),
        Some("DENY")
    );
    assert_eq!(
        resp.headers().get("referrer-policy").and_then(|v| v.to_str().ok()),
        Some("strict-origin-when-cross-origin")
    );
}
