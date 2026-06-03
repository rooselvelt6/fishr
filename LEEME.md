# Fishr 🐟

Sistema de gestión multi-sucursal para pescaderías. Offline-first con sincronización a servidor central.

## Arquitectura

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Sucursal A     │     │  Sucursal B     │     │  Sucursal C     │
│  (SQLite)       │     │  (SQLite)       │     │  (SQLite)       │
│  fishr-agent    │     │  fishr-agent    │     │  fishr-agent    │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │ sync (HTTP)
                         ┌───────┴───────┐
                         │    Central    │
                         │  (PostgreSQL) │
                         │ fishr-central │
                         └───────────────┘
```

## Requisitos

- **Rust** 1.78+ (ver `rust-toolchain.toml`)
- **SQLite** (incluido vía `sqlx`)
- **PostgreSQL** 16+ (solo para central)
- Opcional: **wasm32-unknown-unknown** (para build frontend WASM)

## Estructura del proyecto

```
Cargo.toml              # Workspace con 3 crates
crates/
├── fishr-core/         # Modelos, protocolo de sync, errores
├── fishr-agent/        # Agente de sucursal (Axum + SQLite + Leptos)
└── fishr-central/      # Servidor central (Axum + PostgreSQL)
LEEME.md                # Este archivo
.env.example            # Variables de entorno de ejemplo
docker-compose.yml      # Central + PostgreSQL
rust-toolchain.toml     # Versión de Rust
```

## Instalación y uso

### 1. Clonar y construir

```bash
git clone <repo> && cd fishr
cargo build -p fishr-core
cargo build -p fishr-agent --no-default-features
cargo build -p fishr-central
```

### 2. Configurar sucursal

```bash
cp .env.example .env
# Editar .env con los datos de la sucursal
cargo run -p fishr-agent
```

Esto inicia el servidor web en `http://localhost:8080`.

Primera ejecución: genera automáticamente un `BRANCH_ID` y crea:
- Usuario admin: `admin` / `admin123`
- Métodos de pago (Efectivo, Punto, Transferencia, Pago Móvil, Divisas)
- Preparaciones (Limpieza, Fileteado, Descabezado, Porciones)
- Proveedor de auto-abastecimiento

### 3. Configurar central

```bash
# Requiere PostgreSQL corriendo
export DATABASE_URL="postgres://fishr:fishr@localhost:5432/fishr_central"
cargo run -p fishr-central
```

Servidor central en `http://localhost:9090`.

### 4. Raspberry Pi (kiosk mode)

```bash
chmod +x scripts/install_agent.sh
./scripts/install_agent.sh
```

## Frontend

El frontend usa Leptos SSR (server-side rendering). Para construir con frontend:

```bash
cargo build -p fishr-agent --features frontend --no-default-features
```

Para build WASM (cliente):

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk build crates/fishr-agent/index.html
```

## Desarrollo

### Pruebas

```bash
# Core
cargo test -p fishr-core

# API de agente (SQLite en memoria)
cargo test -p fishr-agent --test api_test --no-default-features

# E2E sync (requiere PostgreSQL)
DATABASE_URL="postgres://fishr:fishr@localhost:5432/fishr_test" \
  cargo test -p fishr-central --test e2e_sync_test -- --ignored
```

### Comandos útiles

```bash
# Build todo el workspace
cargo build --workspace

# Linter
cargo clippy --all-targets

# Formateo
cargo fmt
```

## Variables de entorno

Ver `.env.example` para la lista completa.

| Variable | Descripción | Default |
|---|---|---|
| `BRANCH_ID` | ID único de sucursal (ULID) | — |
| `BRANCH_NAME` | Nombre comercial | Mi Pescadería |
| `BRANCH_RIF` | RIF | J-00000000-0 |
| `CENTRAL_URL` | URL del servidor central | http://localhost:9090 |
| `SCALE_PORT` | Puerto serial de la báscula | /dev/ttyUSB0 |
| `PRINTER_PORT` | Puerto de la impresora | /dev/usb/lp0 |
| `DATABASE_URL` | Conexión PostgreSQL (central) | postgres://fishr:fishr@localhost:5432/fishr_central |

## Licencia

MIT
