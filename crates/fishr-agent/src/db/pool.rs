use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Executor;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect(database_url)
            .await?;

        sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await?;
        sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;

        Ok(Self { pool })
    }

    async fn exec_multi(&self, label: &str, sql: &str) -> anyhow::Result<()> {
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            let clean: Vec<&str> = trimmed
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter(|l| !l.trim().starts_with("--"))
                .collect();
            if clean.is_empty() {
                continue;
            }
            let stmt_str = clean.join(" ");
            if let Err(e) = self.pool.execute(&*stmt_str).await {
                let msg = e.to_string();
                // Legacy: catch duplicate column on ALTER TABLE for migration 005
                // during first upgrade to versioned migrations
                if msg.contains("duplicate column") {
                    tracing::warn!("{} ALTER skipped (already applied): {}", label, msg);
                } else {
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    async fn migration_applied(&self, version: &str) -> anyhow::Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _migrations WHERE version = ?1"
        )
        .bind(version)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    async fn mark_migration(&self, version: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO _migrations (version, name, applied_at) VALUES (?1, ?2, ?3)"
        )
        .bind(version)
        .bind(version)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        let migrations: Vec<(&str, &str)> = vec![
            ("001", include_str!("migrations/001_init.sql")),
            ("002", include_str!("migrations/002_seed.sql")),
            ("003", include_str!("migrations/003_supplier.sql")),
            ("004", include_str!("migrations/004_auth.sql")),
            ("005", include_str!("migrations/005_iva_discount.sql")),
            ("006", include_str!("migrations/006_fish_item_index.sql")),
        ];

        for (version, sql) in &migrations {
            if self.migration_applied(version).await? {
                continue;
            }
            self.exec_multi(version, sql).await?;
            self.mark_migration(version).await?;
            tracing::info!("Migración {} aplicada", version);
        }

        Ok(())
    }

    pub async fn setup_initial_data(&self) -> anyhow::Result<()> {
        let branch_id = ulid::Ulid::new().to_string();
        let now = chrono::Utc::now();

        sqlx::query(
            "INSERT OR IGNORE INTO branch (id, name, address, phone, rif, is_active, op_counter, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
        )
        .bind(&branch_id)
        .bind("Mi Pescadería")
        .bind("Dirección pendiente")
        .bind("0000000000")
        .bind("J-00000000-0")
        .bind(now.timestamp_millis())
        .bind(now)
        .execute(&self.pool)
        .await?;

        let payment_methods = [
            ("Efectivo", "Pago en efectivo"),
            ("Punto de Venta", "Tarjeta de débito/crédito"),
            ("Transferencia", "Transferencia bancaria"),
            ("Pago Móvil", "Pago móvil Bancario"),
            ("Divisas", "Pago en dólares u otras divisas"),
        ];

        for (name, desc) in &payment_methods {
            let id = ulid::Ulid::new().to_string();
            sqlx::query(
                "INSERT OR IGNORE INTO payment_method (id, branch_id, name, description, is_active, op_counter, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6)",
            )
            .bind(&id)
            .bind(&branch_id)
            .bind(name)
            .bind(desc)
            .bind(now.timestamp_millis())
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        let preparations = [
            ("Limpieza básica", "Eviscerado y escamado", 0.50, "Fixed"),
            ("Fileteado", "Filetes sin espinas", 1.00, "Fixed"),
            ("Descabezado", "Pescado sin cabeza", 0.30, "Fixed"),
            ("Cortado en porciones", "Cortado en porciones individuales", 0.75, "Fixed"),
        ];

        for (name, desc, cost, cost_type) in &preparations {
            let id = ulid::Ulid::new().to_string();
            sqlx::query(
                "INSERT OR IGNORE INTO preparation (id, branch_id, name, description, additional_cost, cost_type, is_active, op_counter, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)",
            )
            .bind(&id)
            .bind(&branch_id)
            .bind(name)
            .bind(desc)
            .bind(cost)
            .bind(cost_type)
            .bind(now.timestamp_millis())
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        // Create self-supplier (owner transport)
        let self_supplier = fishr_core::models::Supplier::new_self_supplier(
            branch_id.clone(),
            "Auto-Abastecimiento".into(),
            std::env::var("BRANCH_RIF").unwrap_or_default(),
        );
        sqlx::query(
            "INSERT OR IGNORE INTO supplier (id, branch_id, name, rif, phone, email, address,
             contact_person, is_self, is_active, op_counter, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, 1, ?9, ?10)"
        )
        .bind(&self_supplier.id)
        .bind(&self_supplier.branch_id)
        .bind(&self_supplier.name)
        .bind(&self_supplier.rif)
        .bind(&self_supplier.phone)
        .bind(&self_supplier.email)
        .bind(&self_supplier.address)
        .bind(&self_supplier.contact_person)
        .bind(self_supplier.op_counter)
        .bind(self_supplier.updated_at)
        .execute(&self.pool)
        .await?;

        // Seed default admin user
        let admin_hash = hash_password("admin123")?;
        let admin_id = ulid::Ulid::new().to_string();
        sqlx::query(
            "INSERT OR IGNORE INTO user_account (id, branch_id, username, password_hash, display_name, role, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)"
        )
        .bind(&admin_id)
        .bind(&branch_id)
        .bind("admin")
        .bind(&admin_hash)
        .bind("Administrador")
        .bind("admin")
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Preserve existing .env and append/update BRANCH_ID
        let env_path = std::path::Path::new(".env");
        let mut lines: Vec<String> = if env_path.exists() {
            std::fs::read_to_string(env_path)
                .unwrap_or_default()
                .lines()
                .map(|l| l.to_string())
                .collect()
        } else {
            Vec::new()
        };
        let mut found_branch_id = false;
        for line in &mut lines {
            if line.starts_with("BRANCH_ID=") {
                *line = format!("BRANCH_ID={}", branch_id);
                found_branch_id = true;
            }
        }
        if !found_branch_id {
            lines.push(format!("BRANCH_ID={}", branch_id));
        }
        std::fs::write(".env", lines.join("\n") + "\n")?;

        tracing::info!("Branch ID generated: {}", branch_id);
        Ok(())
    }
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use argon2::Argon2;
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Error al hashear contraseña: {}", e))?;
    Ok(hash.to_string())
}
