# Fishr — Plan de Inteligencia Artificial

## Visión General

Incorporar técnicas de IA evolutiva y bioinspirada para optimizar procesos
del sistema de gestión de pescadería Fishr.

---

## Fase 1 — Asistente de Ventas Inteligente (Sistema Experto Difuso)

**Complejidad:** Baja | **Impacto:** Medio | **Dependencias:** Ninguna

### Descripción
Motor de reglas difusas que analiza el contexto de cada venta en tiempo real
y sugiere preparaciones, descuentos o promociones al cajero.

### Componentes
- **Módulo fuzzy** en `fishr-core`: conjuntos difusos (stock, hora, popularidad),
  motor de reglas IF-THEN, inferencia tipo Mamdani
- **Endpoint `POST /api/pos/suggestions`**: recibe items del carrito + contexto,
  devuelve sugerencias con nivel de confianza
- **Frontend POS**: burbuja de sugerencias al lado del carrito

### Reglas de ejemplo
```
SI stock = ALTO  Y  hora = CIERRE        → sugerir descuento
SI stock = MUY_ALTO  Y  popularidad = BAJA → sugerir promoción
SI cliente = VIP  Y  pescado = PREMIUM    → sugerir preparación premium
SI hora = MAÑANA                          → sugerir captura fresca
SI categoría = BLANCO                     → sugerir fileteado
```

### Archivos a modificar/crear
- `crates/fishr-core/src/fuzzy/` (nuevo módulo)
- `crates/fishr-core/src/lib.rs`
- `crates/fishr-agent/src/api/pos.rs`
- `crates/fishr-agent/src/api/router.rs`
- `crates/fishr-agent/src/frontend/index.html`

---

## Fase 2 — Precios Dinámicos (Lógica Difusa)

**Complejidad:** Media | **Impacto:** Alto | **Dependencias:** Fase 1

### Descripción
Extiende el motor difuso de la Fase 1 para modular precios de mercado en
tiempo real según inventario, hora del día, estacionalidad y demanda histórica.

### Componentes
- **Nuevas reglas difusas**: entrada → factor de ajuste de precio
- **Inyección en `pricing.rs`**: después del cálculo base, aplica el factor
- **Endpoint `GET /api/pricing/suggested`**: vista previa de precios sugeridos
- **UI en inventario**: columna de "Precio sugerido" vs "Precio actual"

### Reglas de ejemplo
```
SI stock = ALTO  Y  hora_semana = TARDE_VIERNES → factor_precio = 0.85
SI stock = BAJO  Y  temporada = ALTA            → factor_precio = 1.10
SI inventario = EXCESO  Y  hora = CIERRE         → factor_precio = 0.70
```

### Archivos a modificar
- `crates/fishr-core/src/fuzzy/sets.rs` (+ PriceFactor, FuzzyInput::new_pos, variables estacionales)
- `crates/fishr-core/src/fuzzy/suggestions.rs` (+ build_pricing_engine, compute_price_factor)
- `crates/fishr-agent/src/api/inventory.rs` (+ GET /api/pricing/suggested)
- `crates/fishr-agent/src/api/pos.rs` (+ precompute_price_factors → inyectado en calculate y confirm)
- `crates/fishr-agent/src/api/router.rs`
- `crates/fishr-agent/src/frontend/index.html`

---

## Fase 3 — Planificador Óptimo de Inventario (Algoritmo Genético)

**Complejidad:** Alta | **Impacto:** Alto | **Dependencias:** Fases 1-2

### Descripción
Algoritmo genético que optimiza las cantidades de compra por especie para
maximizar margen bruto y minimizar desperdicio.

### Componentes
- **Módulo `genetic`** en `fishr-core`: individuo (cantidades por especie),
  población, selección por torneo, cruce uniforme, mutación gaussiana
- **Fitness function**: `margen_bruto - penalización_waste - costo_almacenaje`
- **Tarea agendada**: corre diariamente usando `tokio-cron-scheduler`
  (ya disponible en dependencias)
- **Endpoint `GET /api/planner/suggestions`**: devuelve pedidos sugeridos
- **UI en proveedores**: botón "Cargar sugerencias del planificador" en
  el formulario de recepción

### Flujo
1. El GA carga histórico de ventas (Sale + SaleItem) y precios (MarketPrice)
2. Ejecuta N generaciones hasta converger
3. Devuelve cantidades óptimas por fish_type_id
4. El usuario puede aceptar/modificar antes de crear el SupplierDelivery

### Archivos a crear/modificar
- `crates/fishr-core/src/genetic/` (nuevo módulo)
- `crates/fishr-core/src/lib.rs`
- `crates/fishr-agent/src/api/planner.rs` (nuevo)
- `crates/fishr-agent/src/api/router.rs`
- `crates/fishr-agent/src/main.rs` (tarea agendada)
- Frontend (pestaña "Planificador" en proveedores)

---

## Fase 4 — Ruteo de Preparaciones (Colonia de Hormigas)

**Complejidad:** Alta | **Impacto:** Medio | **Dependencias:** Fase 1-2

### Descripción
Algoritmo ACO (Ant Colony Optimization) para optimizar la secuencia de
preparaciones cuando hay múltiples items en una venta, minimizando el tiempo
total de procesamiento en cocina.

### Componentes
- **Módulo `aco`** en `fishr-core`: grafo de preparaciones, feromonas,
  probabilidad de transición, evaporación, iteraciones
- **Integración en `calculate`**: después de calcular precios, optimiza
  el orden de preparación
- **Output**: secuencia sugerida opcional (no bloqueante)

### Archivos a crear/modificar
- `crates/fishr-core/src/aco/` (nuevo módulo)
- `crates/fishr-core/src/lib.rs`
- `crates/fishr-agent/src/services/pricing.rs`
- `crates/fishr-agent/src/api/pos.rs`

---

## Resumen de Tecnologías

| Fase | Técnica | Tipo | Estado |
|------|---------|------|--------|
| 1 | Sistema Experto Difuso | Fuzzy Logic | ✅ Implementado |
| 2 | Precios Dinámicos | Fuzzy Logic | ✅ Implementado |
| 3 | Planificador de Inventario | Algoritmo Genético | ⬜ Pendiente |
| 4 | Ruteo de Preparaciones | Ant Colony Optimization | ✅ Implementado |
