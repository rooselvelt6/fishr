use std::sync::Arc;
use crate::db::Database;
use crate::sync::SyncConfig;

pub struct AppState {
    pub db: Arc<Database>,
    pub config: AgentConfig,
    pub sync_config: SyncConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub branch_id: String,
    pub branch_name: String,
    pub branch_rif: String,
    pub branch_address: String,
    pub branch_phone: String,
    pub scale_port: Option<String>,
    pub printer_port: Option<String>,
}

impl AgentConfig {
    pub fn from_env() -> Self {
        Self {
            branch_id: std::env::var("BRANCH_ID").unwrap_or_default(),
            branch_name: std::env::var("BRANCH_NAME").unwrap_or_else(|_| "Mi Pescadería".into()),
            branch_rif: std::env::var("BRANCH_RIF").unwrap_or_default(),
            branch_address: std::env::var("BRANCH_ADDRESS").unwrap_or_default(),
            branch_phone: std::env::var("BRANCH_PHONE").unwrap_or_default(),
            scale_port: std::env::var("SCALE_PORT").ok(),
            printer_port: std::env::var("PRINTER_PORT").ok(),
        }
    }
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let config = AgentConfig::from_env();
        let sync_config = SyncConfig::from_env();

        let db = Arc::new(Database::new("sqlite://fishr.db?mode=rwc").await?);
        db.run_migrations().await?;

        Ok(Self {
            db,
            config,
            sync_config,
        })
    }
}
