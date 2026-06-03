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

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        let sql = include_str!("migrations/001_init.sql");
        self.pool.execute(sql).await?;

        let sql = include_str!("migrations/002_seed.sql");
        self.pool.execute(sql).await?;

        let sql = include_str!("migrations/003_supplier.sql");
        self.pool.execute(sql).await?;

        let sql = include_str!("migrations/004_auth.sql");
        self.pool.execute(sql).await?;

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
        let admin_hash = hash_password("admin123");
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

        // Update env file
        std::fs::write(
            ".env",
            format!(
                "BRANCH_ID={}\nBRANCH_RIF=J-00000000-0\nBRANCH_NAME=Mi Pescadería\n",
                branch_id
            ),
        )?;

        tracing::info!("Branch ID generated: {}", branch_id);
        Ok(())
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
