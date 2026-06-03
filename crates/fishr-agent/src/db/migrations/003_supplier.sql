-- 003_supplier.sql
-- Fishr Agent - Supplier & Delivery Management

CREATE TABLE IF NOT EXISTS supplier (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    name TEXT NOT NULL,
    rif TEXT,
    phone TEXT NOT NULL DEFAULT '',
    email TEXT,
    address TEXT,
    contact_person TEXT NOT NULL DEFAULT '',
    is_self INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS supplier_delivery (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branch(id),
    supplier_id TEXT NOT NULL REFERENCES supplier(id),
    supplier_name TEXT NOT NULL,
    delivery_date TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    transport_plate TEXT NOT NULL DEFAULT '',
    transport_driver TEXT NOT NULL DEFAULT '',
    total_cost TEXT NOT NULL DEFAULT '0',
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS supplier_delivery_item (
    id TEXT PRIMARY KEY,
    delivery_id TEXT NOT NULL REFERENCES supplier_delivery(id),
    container_id TEXT NOT NULL REFERENCES container(id),
    container_label TEXT NOT NULL,
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    weight_grams INTEGER NOT NULL,
    unit_cost TEXT NOT NULL DEFAULT '0',
    op_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,
    deleted_at TEXT
);

-- Add supplier columns to fish_item
ALTER TABLE fish_item ADD COLUMN supplier_delivery_item_id TEXT;
ALTER TABLE fish_item ADD COLUMN cost_price TEXT;

CREATE INDEX IF NOT EXISTS idx_supplier_branch ON supplier(branch_id);
CREATE INDEX IF NOT EXISTS idx_supplier_delivery_branch ON supplier_delivery(branch_id);
CREATE INDEX IF NOT EXISTS idx_supplier_delivery_supplier ON supplier_delivery(supplier_id);
CREATE INDEX IF NOT EXISTS idx_supplier_delivery_item_delivery ON supplier_delivery_item(delivery_id);
CREATE INDEX IF NOT EXISTS idx_fish_item_supplier ON fish_item(supplier_delivery_item_id);
