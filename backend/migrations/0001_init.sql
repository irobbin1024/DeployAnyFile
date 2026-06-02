-- Users
CREATE TABLE IF NOT EXISTS users (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT    NOT NULL UNIQUE,
    password_hash TEXT    NOT NULL,
    is_admin      INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT    NOT NULL
);

-- Uploaded files (each file carries one share slug)
CREATE TABLE IF NOT EXISTS files (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id       INTEGER NOT NULL,
    slug          TEXT    NOT NULL UNIQUE,
    original_name TEXT    NOT NULL,
    stored_name   TEXT    NOT NULL,
    mime_type     TEXT    NOT NULL,
    category      TEXT    NOT NULL,
    size          INTEGER NOT NULL,
    is_shared     INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT    NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_files_user ON files (user_id);
CREATE INDEX IF NOT EXISTS idx_files_slug ON files (slug);

-- Visit log for share statistics
CREATE TABLE IF NOT EXISTS visits (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id    INTEGER NOT NULL,
    ip         TEXT    NOT NULL,
    user_agent TEXT,
    visited_at TEXT    NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_visits_file ON visits (file_id);
