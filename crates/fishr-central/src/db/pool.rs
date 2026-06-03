use sqlx::postgres::{PgPool, PgPoolOptions};

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::query(include_str!("migrations/001_init.sql"))
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
