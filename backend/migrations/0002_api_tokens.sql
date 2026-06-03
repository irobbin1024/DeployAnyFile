-- Personal API tokens for scripted uploads (only the hash is stored)
CREATE TABLE IF NOT EXISTS api_tokens (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER NOT NULL,
    name         TEXT    NOT NULL,
    token_hash   TEXT    NOT NULL UNIQUE,
    token_prefix TEXT    NOT NULL,
    created_at   TEXT    NOT NULL,
    last_used_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_api_tokens_user ON api_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_hash ON api_tokens (token_hash);
