//! E2E sync test: agent → central push
//!
//! Requires a running PostgreSQL instance.
//! Run with:
//!   DATABASE_URL="postgres://fishr:fishr@localhost:5432/fishr_test" \
//!     cargo test -p fishr-central --test e2e_sync_test -- --ignored

use fishr_core::sync::{EntityType, SyncPush, SyncRow, SyncResponse};
use std::sync::Arc;

fn is_postgres_available() -> bool {
    std::env::var("DATABASE_URL").is_ok()
}

async fn setup_central_state() -> Option<Arc<fishr_central::Database>> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    let db = Arc::new(fishr_central::Database::new(&database_url).await.ok()?);
    db.run_migrations().await.ok()?;
    Some(db)
}

#[tokio::test]
#[ignore]
async fn test_sync_push_received_by_central() {
    let db = setup_central_state().await.expect("PostgreSQL not available. Set DATABASE_URL and run with --ignored");

    let state = Arc::new(fishr_central::AppState { db: db.clone() });
    let app = fishr_central::build_router(state);

    let now = chrono::Utc::now();
    let row = SyncRow::new(
        EntityType::Branch,
        "e2e-branch-001".into(),
        "e2e-branch-001".into(),
        1,
        now,
        &serde_json::json!({
            "id": "e2e-branch-001",
            "name": "E2E Test Branch",
            "rif": "J-12345678-9",
            "address": "Test Address",
            "phone": "04121234567",
            "is_active": true,
            "op_counter": 1,
        }),
    )
    .expect("Failed to create sync row");

    let push = SyncPush {
        source_branch_id: "e2e-branch-001".into(),
        last_op_counter: 1,
        rows: vec![row],
    };

    let body = serde_json::to_string(&push).unwrap();
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/sync/push")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap();

    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let bytes = resp.collect().await.unwrap().to_bytes();
    let sync_resp: SyncResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(sync_resp.success, "Sync should succeed: {:?}", sync_resp.error);
    assert!(sync_resp.error.is_none(), "No errors expected: {:?}", sync_resp.error);
}

#[tokio::test]
#[ignore]
async fn test_sync_push_invalid_payload() {
    let db = setup_central_state().await.expect("PostgreSQL not available. Set DATABASE_URL and run with --ignored");

    let state = Arc::new(fishr_central::AppState { db: db.clone() });
    let app = fishr_central::build_router(state);

    let body = serde_json::json!({"not_a_valid_push": true}).to_string();
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/sync/push")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap();

    // Should return 422 or 400 for invalid payload
    let resp = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert!(
        resp.status().is_client_error(),
        "Expected client error for invalid payload, got: {}",
        resp.status()
    );
}

#[tokio::test]
#[ignore]
async fn test_sync_push_multiple_rows() {
    let db = setup_central_state().await.expect("PostgreSQL not available. Set DATABASE_URL and run with --ignored");

    let state = Arc::new(fishr_central::AppState { db: db.clone() });
    let app = Arc::new(fishr_central::build_router(state));

    let now = chrono::Utc::now();
    let rows: Vec<SyncRow> = (0..5)
        .map(|i| {
            SyncRow::new(
                EntityType::FishType,
                format!("e2e-ft-{:03}", i),
                "e2e-branch-002".into(),
                i + 1,
                now,
                &serde_json::json!({
                    "id": format!("e2e-ft-{:03}", i),
                    "name": format!("Fish Type {}", i),
                    "species": "Test Species",
                    "category": "White",
                }),
            )
            .unwrap()
        })
        .collect();

    let push = SyncPush {
        source_branch_id: "e2e-branch-002".into(),
        last_op_counter: 5,
        rows,
    };

    let body = serde_json::to_string(&push).unwrap();
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/sync/push")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let bytes = resp.collect().await.unwrap().to_bytes();
    let sync_resp: SyncResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(sync_resp.success);
}

// Helper for body reading
use axum::body::Body;
use http_body_util::BodyExt;
