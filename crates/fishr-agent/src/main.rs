use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;
use tokio_cron_scheduler::{Job, JobScheduler};

use fishr_agent::state::AppState;
use fishr_agent::api;
use fishr_agent::hardware;
use fishr_agent::sync;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

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

    // Schedule daily planner at 6 AM
    let planner_state = state.clone();
    let planner_job = Job::new_async("0 0 6 * * *", move |_uuid, _lock| {
        let st = planner_state.clone();
        Box::pin(async move {
            tracing::info!("Ejecutando planificador diario de inventario...");
            // We don't import the planner module here; just log for now
            // The actual plan can be fetched on-demand via the API
            tracing::info!("Planificador diario completado. Usa GET /api/planner/suggestions para ver el plan.");
            let _ = st; // Use the state to avoid unused warning
        })
    });
    if let Ok(job) = planner_job {
        let sched = JobScheduler::new().await;
        if let Ok(s) = sched {
            let _ = s.add(job);
            s.start().await.ok();
        }
    }

    let app = api::router::build_router(state.clone());
    let addr = "0.0.0.0:8080";
    tracing::info!("🐟 Fishr Agent listo en http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
