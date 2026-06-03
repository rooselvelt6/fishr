use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;
use fishr_central::{api, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let state = Arc::new(AppState::new().await?);
    state.db.run_migrations().await?;

    let app = api::router::build_router(state.clone());
    let addr = "0.0.0.0:9090";
    tracing::info!("🐟 Fishr Central en http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
