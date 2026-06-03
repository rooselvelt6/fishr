-- 001_init.sql
-- Fishr Agent - Database Schema

CREATE TABLE IF NOT EXISTS branch (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    address TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    rif TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS fish_type (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    species TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'White',
    description TEXT NOT NULL DEFAULT '',
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS container (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    fish_type_id TEXT NOT NULL REFERENCES fish_type(id),
    fish_type_name TEXT NOT NULL,
    label TEXT NOT NULL,
    capacity INTEGER NOT NULL DEFAULT 50,
    current_count INTEGER NOT NULL DEFAULT 0,
    location TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS fish_item (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    container_id TEXT NOT NULL REFERENCES container(id),
    container_label TEXT NOT NULL,
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    weight_grams INTEGER NOT NULL,
    added_at TEXT NOT NULL,
    sold_at TEXT,
    sold_in_sale_id TEXT,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS customer (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    name TEXT NOT NULL,
    phone TEXT NOT NULL DEFAULT '',
    email TEXT,
    rif TEXT,
    address TEXT,
    points INTEGER NOT NULL DEFAULT 0,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS payment_method (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_active INTEGER NOT NULL DEFAULT 1,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS preparation (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    additional_cost REAL NOT NULL DEFAULT 0.0,
    cost_type TEXT NOT NULL DEFAULT 'Fixed',
    is_active INTEGER NOT NULL DEFAULT 1,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS market_price (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    price_per_kg REAL NOT NULL,
    cost_price REAL NOT NULL DEFAULT 0.0,
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS sale (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    customer_id TEXT REFERENCES customer(id),
    customer_name TEXT,
    payment_method_id TEXT NOT NULL,
    payment_method_name TEXT NOT NULL,
    subtotal REAL NOT NULL DEFAULT 0.0,
    preparation_fee REAL NOT NULL DEFAULT 0.0,
    total REAL NOT NULL DEFAULT 0.0,
    item_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS sale_item (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    sale_id TEXT NOT NULL REFERENCES sale(id),
    fish_item_id TEXT NOT NULL,
    container_id TEXT NOT NULL,
    container_label TEXT NOT NULL,
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    weight_grams INTEGER NOT NULL,
    price_per_kg REAL NOT NULL,
    preparation_id TEXT,
    preparation_name TEXT,
    preparation_fee REAL NOT NULL DEFAULT 0.0,
    subtotal REAL NOT NULL DEFAULT 0.0,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS invoice (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    sale_id TEXT NOT NULL REFERENCES sale(id),
    customer_id TEXT,
    customer_name TEXT,
    customer_rif TEXT,
    customer_address TEXT,
    control_number TEXT NOT NULL,
    total REAL NOT NULL,
    issued_at TEXT NOT NULL,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS pending_sync (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    branch_id TEXT NOT NULL,
    op_counter INTEGER NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL,
    synced_at TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sync_log (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    op_counter INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    synced_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_fish_item_available ON fish_item(sold_at) WHERE sold_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_fish_item_container ON fish_item(container_id);
CREATE INDEX IF NOT EXISTS idx_sale_branch ON sale(branch_id, created_at);
CREATE INDEX IF NOT EXISTS idx_sale_item_sale ON sale_item(sale_id);
CREATE INDEX IF NOT EXISTS idx_pending_sync_synced ON pending_sync(synced_at);
CREATE INDEX IF NOT EXISTS idx_pending_sync_op ON pending_sync(op_counter);
CREATE INDEX IF NOT EXISTS idx_container_branch ON container(branch_id);
CREATE INDEX IF NOT EXISTS idx_market_price_fish ON market_price(fish_type_id);
CREATE INDEX IF NOT EXISTS idx_invoice_sale ON invoice(sale_id);
