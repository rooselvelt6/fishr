# Fishr — Plan de Hardening (5 Fases)

## Visión General

Refactorización integral del sistema Fishr para llevarlo a calidad de producción.
Basado en auditoría de arquitectura, seguridad y calidad de código Rust.

---

## Fase 1 — Seguridad y Bugs Críticos

**Complejidad:** Media | **Impacto:** Crítico | **Dependencias:** Ninguna

### 1.1 Auth middleware no valida token

**Problema:** `rate_limit.rs:68-83` solo verifica que `x-session-token` no esté vacío.
Cualquier string funciona para rutas protegidas que no usen el extractor `AuthUser`.

**Solución:** Validar token contra DB en el middleware `auth_required`, o al menos
rechazar si `AuthUser::from_request_parts` falla.

**Archivos:**
- `crates/fishr-agent/src/api/rate_limit.rs`
- `crates/fishr-agent/src/api/auth.rs`
- `crates/fishr-agent/src/api/router.rs`

### 1.2 `hash_password` traga errores

**Problema:** `pool.rs:203` — si Argon2 falla, guarda `""` como hash, lockeando al admin.

**Solución:** Propagar el error con `?` en lugar de `unwrap_or_else`.

**Archivos:**
- `crates/fishr-agent/src/db/pool.rs`

### 1.3 Sesiones sin expiración en middleware

**Problema:** No se verifica expiración de sesión en el middleware; la validación
se delega a handlers individuales.

**Solución:** Agregar verificación de `expires_at` en `auth_required`.

**Archivos:**
- `crates/fishr-agent/src/api/rate_limit.rs`
- `crates/fishr-agent/src/api/auth.rs`

---

## Fase 2 — Base de Datos y Migraciones

**Complejidad:** Alta | **Impacto:** Alto | **Dependencias:** Fase 1

### 2.1 Sistema de migraciones sin versioning

**Problema:** `pool.rs:49-57` re-ejecuta migraciones en cada startup con
`include_str!`. Usa `CREATE TABLE IF NOT EXISTS` y detecta `duplicate column`
por string match — extremadamente frágil.

**Solución:** Implementar tabla `_migrations` con versioning, o migrar a
`sqlx migrate` con archivos de migración numerados y `sqlx migrate run`.

**Archivos:**
- `crates/fishr-agent/src/db/pool.rs`
- `crates/fishr-agent/src/db/migrations/` (reestructurar)

### 2.2 Tipos decimales inconsistentes

**Problema:** `001_init.sql` usa `REAL` para `sale.subtotal`/`sale.total`,
pero `005_iva_discount.sql` añade `tax_amount TEXT` y `discount_amount TEXT`.
Mezcla flotantes imprecisos con texto.

**Solución:** Unificar todo a `TEXT` (almacenado como string de rust_decimal)
o a `REAL` con redondeo explícito. Preferir `TEXT` para precisión monetaria.

**Archivos:**
- Migraciones SQL involucradas
- Modelos y handlers que usan esos campos

### 2.3 Falta FK de `sale_item.fish_item_id`

**Problema:** `sale_item.fish_item_id TEXT NOT NULL` sin `REFERENCES fish_item(id)`.

**Solución:** Agregar la FK o documentar por qué se omitió intencionalmente.

**Archivos:**
- Migración correspondiente

### 2.4 Seed data con IDs hardcodeados

**Problema:** `002_seed.sql` usa IDs como `'ft_default_001'` que no son ULIDs.

**Solución:** Usar ULIDs generados o mantener IDs fijos pero documentados.

**Archivos:**
- `crates/fishr-agent/src/db/migrations/002_seed.sql`

---

## Fase 3 — Manejo de Errores y Robustez

**Complejidad:** Media | **Impacto:** Alto | **Dependencias:** Fase 1

### 3.1 Error swallowing en parseo de fechas

**Problema:** Múltiples `into_model()` usan `unwrap_or_default()` en parseo de
datetimes, devolviendo época UNIX (1970) silenciosamente.

**Solución:** Propagar el error con `?` o loguear con `tracing::warn!`. Ideal:
que `from_row` retorne `Result<T, ApiError>`.

**Archivos:**
- Todos los `impl IntoModel` / `FromRow` con parseo de fechas

### 3.2 `Result<(), String>` en sync agent

**Problema:** `sync/agent.rs:22` retorna `Result<(), String>`. Los call sites
hacen `e.starts_with("Connection")` — frágil y pierde tipo.

**Solución:** Crear enum `SyncError` con variantes tipadas.

**Archivos:**
- `crates/fishr-agent/src/sync/agent.rs`
- `crates/fishr-agent/src/sync/mod.rs`

### 3.3 `anyhow` mezclado con `ApiError`

**Problema:** Tres sistemas de error conviven: `CoreError`, `ApiError`, `anyhow`.

**Solución:** Definir boundaries claros. `anyhow` solo para bootstrap/main.
`ApiError` para handlers. `CoreError` para lógica de dominio.

**Archivos:**
- `crates/fishr-agent/src/state.rs`
- `crates/fishr-agent/src/db/pool.rs`

### 3.4 `category_to_string` con JSON hack

**Problema:** `inventory.rs` usa `serde_json::to_string()` + `trim_matches('"')`
para convertir categoría a string.

**Solución:** Implementar `Display` o `as_str()` en `FishCategory`.

**Archivos:**
- `crates/fishr-agent/src/api/inventory.rs`
- `crates/fishr-core/src/models/fish_type.rs`

---

## Fase 4 — Arquitectura y Calidad de Código

**Complejidad:** Alta | **Impacto:** Medio | **Dependencias:** Fase 2

### 4.1 `SyncConfig` duplicado

**Problema:** Definición idéntica en `fishr-core/src/sync/protocol.rs:62-70` y
`fishr-agent/src/sync/mod.rs:4-11`. Si se añade un campo a uno, el otro queda
inconsistente.

**Solución:** Unificar en `fishr-core`. El core define el struct con `Default`;
el agent implementa `from_env()` usando extensión o constructor.

**Archivos:**
- `crates/fishr-core/src/sync/protocol.rs`
- `crates/fishr-core/src/sync/mod.rs`
- `crates/fishr-agent/src/sync/mod.rs`
- `crates/fishr-agent/src/sync/agent.rs`

### 4.2 Trait `SyncEntity` definido pero no implementado

**Problema:** `sync_meta.rs:5-14` define el trait pero ningún modelo lo
implementa. Código muerto.

**Solución:** Implementar `SyncEntity` en todos los modelos sync, o remover
el trait si no se necesita.

**Archivos:**
- `crates/fishr-core/src/models/sync_meta.rs`
- Todos los modelos sync

### 4.3 `push_sync` en módulo incorrecto

**Problema:** `push_sync()` vive en `inventory.rs:706` pero lo llaman `auth.rs`,
`setup.rs`, etc. Layering violation.

**Solución:** Mover a `crate::sync::mod.rs` o a un helper compartido.

**Archivos:**
- `crates/fishr-agent/src/api/inventory.rs`
- `crates/fishr-agent/src/sync/mod.rs`
- Todos los call sites

### 4.4 Row types duplicados en cada handler

**Problema:** Cada módulo API define sus propios `*Row` structs con
`#[derive(sqlx::FromRow)]` y `into_model()` idénticos.

**Solución:** Centralizar en `fishr-core` junto a modelos, o usar `query_as`
directamente con los modelos si los nombres de columna coinciden.

**Archivos:**
- Múltiples módulos API

### 4.5 Glob re-exports en models

**Problema:** `pub use module::*` en `models/mod.rs` hace el API público opaco.

**Solución:** Usar `pub use module::{ModelA, ModelB}` explícito.

**Archivos:**
- `crates/fishr-core/src/models/mod.rs`

### 4.6 `inventory.rs` con 728 líneas

**Problema:** Mezcla handlers, row types, lógica fuzzy, y helper de sync.

**Solución:** Extraer `pricing.rs` como módulo separado, mover row types a
sus modelos, mover `push_sync` a sync module.

**Archivos:**
- `crates/fishr-agent/src/api/inventory.rs`
- `crates/fishr-agent/src/api/pricing.rs` (nuevo)

---

## Fase 5 — Performance y Hardening

**Complejidad:** Baja | **Impacto:** Medio | **Dependencias:** Fase 2

### 5.1 N+1 query en motor de precios

**Problema:** `inventory.rs:658-665` hace COUNT por fish_type dentro de un loop.

**Solución:** Batch query con `GROUP BY`.

**Archivos:**
- `crates/fishr-agent/src/api/inventory.rs` (o `pricing.rs`)

### 5.2 Sin paginación en list endpoints

**Problema:** `list_fish_types`, `list_containers`, `list_customers`, `list_sales`,
etc. retornan todas las filas sin LIMIT/OFFSET.

**Solución:** Agregar `?limit` y `?offset` opcionales con defaults sensatos.

**Archivos:**
- Múltiples handlers API

### 5.3 Batch `push_sync` en inserciones múltiples

**Problema:** Al insertar N fish items, `push_sync` se llama N veces (N inserts
a `pending_sync`).

**Solución:** Colectar en Vec y hacer un solo INSERT multi-row.

**Archivos:**
- `crates/fishr-agent/src/api/inventory.rs`

### 5.4 Sync retry sin exponential backoff

**Problema:** En fallo, espera `retry_delay_secs` fijo (60s) y reintenta a máxima
velocidad. Un endpoint caído recibe hammering.

**Solución:** Implementar backoff exponencial: `delay * 2^n` hasta un máximo.

**Archivos:**
- `crates/fishr-agent/src/sync/agent.rs`

### 5.5 `dotenvy::dotenv()` en state, no en main

**Problema:** `state.rs:39` carga `.env` al crear AppState, no al iniciar el
programa.

**Solución:** Mover a `main.rs` antes de cualquier inicialización.

**Archivos:**
- `crates/fishr-agent/src/state.rs`
- `crates/fishr-agent/src/main.rs`

### 5.6 Hardcoded SQLite URL

**Problema:** `"sqlite://fishr.db?mode=rwc"` hardcodeado en `state.rs:43`.

**Solución:** Leer de `DATABASE_URL` env var con ese valor como default.

**Archivos:**
- `crates/fishr-agent/src/state.rs`
- `.env.example`

### 5.7 `op_counter` usa wall clock

**Problema:** `timestamp_millis()` como op_counter es vulnerable a clock skew,
NTP ajustes, leap seconds.

**Solución:** Usar contador monótono por branch, o HLC (Hybrid Logical Clock).

**Archivos:**
- `crates/fishr-core/src/sync/sync_meta.rs`
- `crates/fishr-agent/src/sync/agent.rs`

### 5.8 `unwrap()` en reqwest Client

**Problema:** `main.rs:17` — `reqwest::Client::builder()...build().unwrap()`.
Paniquea si TLS falla.

**Solución:** Propagar con `?`.

**Archivos:**
- `crates/fishr-agent/src/main.rs`

---

## Resumen

| Fase | Enfoque | Impacto | Dependencias |
|------|---------|---------|--------------|
| 1 | Seguridad y bugs críticos | Crítico | Ninguna |
| 2 | Base de datos y migraciones | Alto | Fase 1 |
| 3 | Manejo de errores y robustez | Alto | Fase 1 |
| 4 | Arquitectura y calidad de código | Medio | Fase 2 |
| 5 | Performance y hardening | Medio | Fase 2 |

**Prioridad:** Ejecutar en orden secuencial. Fase 1 es prerequisito para
producción. Fases 2-3 son calidad de datos y resiliencia. Fases 4-5 son
mantenibilidad a largo plazo.
