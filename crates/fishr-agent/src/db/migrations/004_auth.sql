CREATE TABLE IF NOT EXISTS user_account (
    id TEXT PRIMARY KEY,
    branch_id TEXT NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    role TEXT NOT NULL DEFAULT 'admin' CHECK(role IN ('admin', 'cajero')),
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_session (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES user_account(id),
    token_hash TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_session_token ON user_session(token_hash);
CREATE INDEX IF NOT EXISTS idx_user_session_user ON user_session(user_id);
CREATE INDEX IF NOT EXISTS idx_user_account_username ON user_account(username);
