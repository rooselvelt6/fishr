pub mod agent;

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub central_url: String,
    pub branch_id: String,
    pub sync_interval_secs: u64,
    pub max_batch_size: usize,
    pub retry_delay_secs: u64,
    pub max_retries: i32,
}

impl SyncConfig {
    pub fn from_env() -> Self {
        Self {
            central_url: std::env::var("CENTRAL_URL").unwrap_or_default(),
            branch_id: std::env::var("BRANCH_ID").unwrap_or_default(),
            sync_interval_secs: std::env::var("SYNC_INTERVAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            max_batch_size: std::env::var("SYNC_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            retry_delay_secs: std::env::var("SYNC_RETRY_DELAY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
            max_retries: std::env::var("SYNC_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        }
    }
}
