use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use fishr_agent::state::AppState;
use fishr_agent::api;
use fishr_agent::hardware;
use fishr_agent::sync;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let state = Arc::new(AppState::new().await?);
    state.db.run_migrations().await?;

    if state.config.branch_id.is_empty() {
        tracing::warn!("BRANCH_ID not set, running first-time setup...");
        state.db.setup_initial_data().await?;
    }

    if let Some(port) = &state.config.scale_port {
        hardware::scale::init_scale(port, 9600);
        tracing::info!("Báscula inicializada en {}", port);
    }
    if let Some(port) = &state.config.printer_port {
        hardware::printer::init_printer(port, 19200);
        tracing::info!("Impresora inicializada en {}", port);
    }

    let sync_config = state.sync_config.clone();
    if !sync_config.central_url.is_empty() {
        let central_url = sync_config.central_url.clone();
        let sync_state = state.clone();
        tracing::info!("Sync agent iniciado hacia {}", central_url);
        tokio::spawn(async move {
            sync::agent::run_sync_loop(sync_state, sync_config).await;
        });
    }

    let app = api::router::build_router(state.clone());
    let addr = "0.0.0.0:8080";
    tracing::info!("🐟 Fishr Agent listo en http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
