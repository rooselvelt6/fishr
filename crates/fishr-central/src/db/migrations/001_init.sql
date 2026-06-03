CREATE TABLE IF NOT EXISTS branches (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    address TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    rif TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT true,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS fish_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    species TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'White',
    description TEXT NOT NULL DEFAULT '',
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS containers (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    label TEXT NOT NULL,
    capacity INTEGER NOT NULL DEFAULT 50,
    current_count INTEGER NOT NULL DEFAULT 0,
    location TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT true,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS fish_items (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    container_id TEXT NOT NULL,
    container_label TEXT NOT NULL,
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    weight_grams INTEGER NOT NULL,
    added_at TIMESTAMPTZ NOT NULL,
    sold_at TIMESTAMPTZ,
    sold_in_sale_id TEXT,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS customers (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    name TEXT NOT NULL,
    phone TEXT NOT NULL DEFAULT '',
    email TEXT,
    rif TEXT,
    address TEXT,
    points BIGINT NOT NULL DEFAULT 0,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS payment_methods (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT true,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS preparations (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    additional_cost NUMERIC NOT NULL DEFAULT 0.0,
    cost_type TEXT NOT NULL DEFAULT 'Fixed',
    is_active BOOLEAN NOT NULL DEFAULT true,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS market_prices (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    price_per_kg NUMERIC NOT NULL,
    cost_price NUMERIC NOT NULL DEFAULT 0.0,
    effective_from TIMESTAMPTZ NOT NULL,
    effective_to TIMESTAMPTZ,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sales (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    customer_id TEXT,
    customer_name TEXT,
    payment_method_id TEXT NOT NULL,
    payment_method_name TEXT NOT NULL,
    subtotal NUMERIC NOT NULL DEFAULT 0.0,
    preparation_fee NUMERIC NOT NULL DEFAULT 0.0,
    total NUMERIC NOT NULL DEFAULT 0.0,
    item_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sale_items (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    sale_id TEXT NOT NULL REFERENCES sales(id),
    fish_item_id TEXT NOT NULL,
    container_id TEXT NOT NULL,
    container_label TEXT NOT NULL,
    fish_type_id TEXT NOT NULL,
    fish_type_name TEXT NOT NULL,
    weight_grams INTEGER NOT NULL,
    price_per_kg NUMERIC NOT NULL,
    preparation_id TEXT,
    preparation_name TEXT,
    preparation_fee NUMERIC NOT NULL DEFAULT 0.0,
    subtotal NUMERIC NOT NULL DEFAULT 0.0,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL REFERENCES branches(id),
    sale_id TEXT NOT NULL REFERENCES sales(id),
    customer_id TEXT,
    customer_name TEXT,
    customer_rif TEXT,
    customer_address TEXT,
    control_number TEXT NOT NULL,
    total NUMERIC NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL,
    op_counter BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sync_log (
    id TEXT PRIMARY KEY,
    source_branch_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    op_counter BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_central_sales_branch ON sales(branch_id, created_at);
CREATE INDEX IF NOT EXISTS idx_central_sync_log_source ON sync_log(source_branch_id);
CREATE INDEX IF NOT EXISTS idx_central_containers_branch ON containers(branch_id);
CREATE INDEX IF NOT EXISTS idx_central_fish_branch ON fish_items(branch_id);
