use crate::auth::hash_password;
use crate::config::Config;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

pub async fn init_pool(config: &Config) -> anyhow::Result<SqlitePool> {
    // Strip the sqlite:// scheme to get a filesystem path for create_if_missing.
    let raw = config
        .database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");

    if let Some(parent) = std::path::Path::new(raw).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let options = SqliteConnectOptions::new()
        .filename(raw)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Ensure the configured super-admin account exists.
pub async fn bootstrap_admin(pool: &SqlitePool, config: &Config) -> anyhow::Result<()> {
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = ?")
            .bind(&config.admin_username)
            .fetch_optional(pool)
            .await?;

    if existing.is_none() {
        let hash = hash_password(&config.admin_password)
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        sqlx::query("INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, 1, ?)")
            .bind(&config.admin_username)
            .bind(&hash)
            .bind(Utc::now().to_rfc3339())
            .execute(pool)
            .await?;
        tracing::info!("created super admin '{}'", config.admin_username);
    }
    Ok(())
}
