pub mod api;
pub mod db;
pub mod frontend;
pub mod sync;

use std::sync::Arc;

pub use api::router::build_router;
pub use db::Database;

pub struct AppState {
    pub db: Arc<Database>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://fishr:fishr@localhost:5432/fishr_central".into());
        let db = Arc::new(Database::new(&database_url).await?);
        Ok(Self { db })
    }
}
