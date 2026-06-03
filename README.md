# Fishr 🐟

Sistema de gestión multi-sucursal para pescaderías. **Offline-first** con sincronización periódica a servidor central. Construido en Rust.

## Arquitectura

```
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│  Sucursal A         │     │  Sucursal B         │     │  Sucursal C         │
│  fishr-agent        │     │  fishr-agent        │     │  fishr-agent        │
│  SQLite local       │     │  SQLite local       │     │  SQLite local       │
│  Puerto 8080        │     │  Puerto 8080        │     │  Puerto 8080        │
└──────────┬──────────┘     └──────────┬──────────┘     └──────────┬──────────┘
           │                          │                          │
           └──────────────────────────┼──────────────────────────┘
                                      │ sync (HTTP/JSON)
                              ┌───────┴───────┐
                              │   Central     │
                              │  PostgreSQL   │
                              │ fishr-central │
                              │  Puerto 9090  │
                              └───────────────┘
```

### Principios

- **Offline-first**: cada sucursal opera con SQLite local 100% del tiempo. Si no hay conexión al central, el trabajo continúa sin interrupción.
- **Sincronización asíncrona**: cola de pendientes con reintentos. Cada operación genera un `op_counter` monotónico por sucursal.
- **IDs globales ULID**: identificadores únicos sin coordinación central. Ordenables por tiempo.
- **Last-writer-wins**: en conflicto, gana el `op_counter` más alto.

### Flujo de caja (POS)

```
Seleccionar pescado → Báscula lee peso → Preparación opcional → Calcular total
  → Seleccionar método de pago → Confirmar venta → Imprimir factura → Sync
```

### Cobertura funcional

| Módulo | Funcionalidad |
|---|---|
| Punto de Venta (POS) | Peso por báscula, preparaciones, cálculo de precios, factura |
| Inventario | Tipos de pescado, contenedores/capacidad, precios de mercado |
| Clientes | Registro, búsqueda, historial de compras |
| Proveedores | Auto-abastecimiento, entregas con generación automática de inventario |
| Reportes | Ventas diarias, desglose por hora, productos top, valorización de inventario |
| Sincronización | Cola de pendientes, push periódico, estado del sync |
| Autenticación | Sesiones con Argon2id + SHA-256, roles admin/cajero |

## Tecnología

### Stack principal

| Capa | Tecnología |
|---|---|
| Lenguaje | **Rust** 1.96 (edition 2021) |
| Web framework | **Axum** 0.8 |
| Base de datos sucursal | **SQLite** via `sqlx` 0.8 (WAL mode, 4 conexiones) |
| Base de datos central | **PostgreSQL** 16+ via `sqlx` 0.8 (10 conexiones) |
| Frontend | **Leptos** 0.7 SSR + WASM + Tailwind CSS + Font Awesome |
| Sync HTTP | **reqwest** 0.12 |
| Serialización | **serde** + **serde_json** |
| IDs | **ulid** |
| Decimales | **rust_decimal** (precios monetarios) |
| Hashing | **argon2** (contraseñas) + **sha2** (tokens) |
| Procesamiento paralelo | **Rayon** 1.10 |
| Tareas programadas | **tokio-cron-scheduler** |
| Puerto serial | **serialport** 4.3 (báscula) |
| Limpieza memoria | **zeroize** |
| Logging | **tracing** + **tracing-subscriber** |

### Estructura del workspace

```
Cargo.toml                        # Workspace raíz (3 crates)
├── crates/fishr-core/            # Biblioteca compartida
│   ├── src/models/               #   14 modelos de dominio
│   └── src/sync/                 #   Protocolo de sincronización
├── crates/fishr-agent/           # Agente de sucursal
│   ├── src/api/                  #   Handlers Axum (11 módulos)
│   ├── src/db/migrations/        #   4 migraciones SQL
│   ├── src/frontend/             #   Leptos SSR (opcional, feature)
│   ├── src/hardware/             #   Báscula + impresora (opcional, feature)
│   ├── src/sync/                 #   Cliente de sync
│   └── src/services/             #   Pricing + reportes con Rayon
├── crates/fishr-central/         # Servidor central
│   ├── src/api/                  #   Sync receiver + dashboard
│   ├── src/db/migrations/        #   Esquema PostgreSQL
│   └── src/frontend/             #   Dashboard central Leptos
├── scripts/                      #   Instaladores PC y Raspberry Pi
├── hardware/scales/              #   Documentación protocolo básculas
├── .github/workflows/ci.yml      #   CI pipeline
└── docker-compose.yml            #   Central + PostgreSQL
```

## Pruebas

### Suite de pruebas

| Suite | Tipo | Cantidad | Comando |
|---|---|---|---|
| fishr-core | Unitarias | 11 | `cargo test -p fishr-core` |
| fishr-agent | Integración API | 14 | `cargo test -p fishr-agent --test api_test --no-default-features` |
| fishr-central | E2E sync | 3 (ignoradas por defecto) | `DATABASE_URL=... cargo test -p fishr-central --test e2e_sync_test -- --ignored` |

### Pruebas de integración (API)

Cubren el flujo completo del agente con SQLite en memoria:

| # | Prueba | Verifica |
|---|---|---|
| 1 | `test_health_check` | Endpoint `/api/health` responde 200 |
| 2 | `test_login_wrong_password` | Rechazo con credenciales inválidas |
| 3 | `test_login_success` | Login devuelve token + usuario |
| 4 | `test_me_without_token` | Bloqueo sin token |
| 5 | `test_me_with_valid_token` | Perfil de usuario con token válido |
| 6 | `test_logout_clears_session` | Logout invalida el token |
| 7 | `test_protected_route_requires_auth` | Ruta protegida sin token → 401 |
| 8 | `test_list_payment_methods` | Métodos de pago seed |
| 9 | `test_list_containers_empty` | Lista vacía de contenedores |
| 10 | `test_create_and_list_containers` | CRUD contenedores |
| 11 | `test_create_customer` | Creación de cliente |
| 12 | `test_create_customer_validation` | Validación de campos vacíos |
| 13 | `test_pos_confirm_sale` | Flujo POS completo (items → venta → factura) |
| 14 | `test_security_headers_present` | Headers de seguridad en todas las respuestas |

### Pruebas E2E de sincronización (requieren PostgreSQL)

| # | Prueba | Verifica |
|---|---|---|
| 1 | `test_sync_push_received_by_central` | Push de fila desde sucursal |
| 2 | `test_sync_push_invalid_payload` | Payload inválido → 4xx |
| 3 | `test_sync_push_multiple_rows` | Push de 5 filas en lote |

## Análisis de velocidad y rendimiento

### Procesamiento paralelo con Rayon

Los reportes aprovechan Rayon para paralelizar operaciones sobre conjuntos de datos:

- **Reporte diario** (`daily_report`): cálculo paralelo de ingresos totales, ingresos por método de pago, productos más vendidos, y desglose por hora — 4 operaciones `par_iter()` concurrentes.
- **Valorización de inventario** (`inventory_valuation`): suma paralela del valor total del inventario (`par_iter().sum()`).
- **Reportes consolidados** (`reporting.rs`): agregación paralela de revenue total, cantidad de items, y productos por hora.

### Configuración de sincronización

| Parámetro | Default | Descripción |
|---|---|---|
| `SYNC_INTERVAL` | 300s (5 min) | Intervalo entre ciclos de sync |
| `SYNC_BATCH_SIZE` | 100 | Máximo de filas por push |
| `SYNC_RETRY_DELAY` | 60s | Espera antes de reintentar |
| `SYNC_MAX_RETRIES` | 10 | Reintentos máximos por fila |
| `CENTRAL_URL` | http://localhost:9090 | URL del servidor central |

### Rate limiting

| Endpoint | Límite | Ventana | Ubicación |
|---|---|---|---|
| Login (`/api/auth/login`) | 10 req | 60 segundos por IP | `fishr-agent` |
| Sync (`/api/sync/push`) | 100 req | 60 segundos por IP | `fishr-central` |

### Pool de conexiones

- **SQLite (sucursal)**: 4 conexiones máx., WAL journal mode, foreign keys ON
- **PostgreSQL (central)**: 10 conexiones máx.

### Comandos de build

```bash
# Build individual
cargo build -p fishr-core
cargo build -p fishr-agent --no-default-features
cargo build -p fishr-agent
cargo build -p fishr-agent --features frontend --no-default-features
cargo build -p fishr-central

# Release
cargo build -p fishr-agent --release
cargo build -p fishr-central --release

# Tests
cargo test -p fishr-core
cargo test -p fishr-agent --test api_test --no-default-features
cargo test -p fishr-agent --test api_test

# Linting
cargo clippy --all-targets
cargo fmt
```

## Seguridad

### Autenticación

- **Sin JWT**: sesiones con token aleatorio de 64 caracteres (`OsRng`).
- **Tokens hasheados**: SHA-256 antes de almacenar en DB.
- **Contraseñas**: Argon2id con salt generado por `OsRng`.
- **Expiración**: sesiones válidas por 12 horas.
- **Roles**: `admin` y `cajero` (validado con `CHECK` constraint en SQL).
- **Middleware `AuthUser`**: extractor de Axum que valida `x-session-token` contra DB en cada request protegido.

### Rate limiting

- **Login**: 10 intentos por minuto por IP usando `SlidingWindowCounter` en memoria.
- **Sync central**: 100 requests por minuto por IP.

### Headers de seguridad

Aplicados en todas las respuestas de ambos servidores:

| Header | Valor |
|---|---|
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | `geolocation=(), microphone=(), camera=()` |

### CORS

Orígenes permitidos (sin wildcard):
```
http://localhost:8080
http://127.0.0.1:8080
http://localhost:9090   (solo central)
http://127.0.0.1:9090   (solo central)
```

### Zeroize (memoria sensible)

- Contraseñas limpiadas con `zeroize()` después de verificar.
- Buffer de generación de tokens limpiado con `zeroize()`.

### Validación de entrada

Sanitizadores en `crates/fishr-agent/src/api/error.rs`:

| Validador | Regla |
|---|---|
| `validate_not_empty` | String no vacío |
| `validate_positive_i32` | Entero positivo (> 0) |
| `validate_weight` | Peso entre 0.001 y 9999.999 kg |
| `validate_non_negative_f64` | Decimal no negativo |

### Manejo de errores

- `sqlx::Error::RowNotFound` → 404
- Otros errores de DB → 500 (sin filtrar información interna)
- Tipos `ApiError`: `bad_request`, `not_found`, `internal`, `conflict`, `unauthorized`

### Sincronización segura

- Conflicto resuelto por `op_counter`: `WHERE tabla.op_counter < EXCLUDED.op_counter`
- Log de toda actividad de sync para auditoría

## Protocolo de sincronización

### Entidades sincronizables (16)

```
Branch, FishType, Container, FishItem, Customer, Sale, SaleItem,
MarketPrice, PaymentMethod, Preparation, Invoice, Supplier,
SupplierDelivery, SupplierDeliveryItem
```

### Flujo

```
Sucursal                                Central
   │                                       │
   │── POST /api/sync/push ───────────►    │
   │   { source_branch_id,                 │
   │     last_op_counter,                  │
   │     rows: [SyncRow, ...] }            │
   │                                       │── UPSERT por op_counter
   │                                       │── Log a sync_log
   │◄── SyncResponse ─────────────────     │
   │   { success, new_op_counter,          │
   │     server_updates, error }           │
```

### Cola de pendientes

Cada operación de inventario, venta o factura genera una entrada `PendingSync` local. El `SyncAgent` periódicamente:
1. Obtiene hasta `max_batch_size` filas ordenadas por `op_counter`
2. Las convierte a `SyncRow` y envía POST al central
3. En éxito: marca como `synced_at IS NOT NULL`
4. En fallo: incrementa `retry_count`, reintenta hasta `max_retries`

## Variables de entorno

Ver `.env.example` para la lista completa.

| Variable | Descripción | Default |
|---|---|---|
| `BRANCH_ID` | ID único de sucursal (ULID) | — |
| `BRANCH_NAME` | Nombre comercial | Mi Pescadería |
| `BRANCH_RIF` | RIF | J-00000000-0 |
| `CENTRAL_URL` | URL del servidor central | http://localhost:9090 |
| `SCALE_PORT` | Puerto serial de la báscula | /dev/ttyUSB0 |
| `SCALE_MODE` | Modo báscula (`continuous` o `command`) | continuous |
| `SCALE_BAUD` | Baud rate | 9600 |
| `PRINTER_PORT` | Puerto de la impresora | /dev/usb/lp0 |
| `DATABASE_URL` | Conexión PostgreSQL (central) | postgres://fishr:fishr@localhost:5432/fishr_central |
| `SYNC_INTERVAL` | Intervalo de sync (segundos) | 300 |
| `SYNC_BATCH_SIZE` | Máximo filas por push | 100 |

## Licencia

MIT
