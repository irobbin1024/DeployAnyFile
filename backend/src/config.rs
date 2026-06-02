use std::env;

#[derive(Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub upload_dir: String,
    pub jwt_secret: String,
    pub admin_username: String,
    pub admin_password: String,
    pub max_upload_mb: usize,
    pub static_dir: String,
}

fn var_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn from_env() -> Self {
        let data_dir = var_or("DATA_DIR", "./data");
        Config {
            bind_addr: var_or("BIND_ADDR", "0.0.0.0:8080"),
            database_url: var_or("DATABASE_URL", &format!("sqlite://{data_dir}/app.db")),
            upload_dir: var_or("UPLOAD_DIR", &format!("{data_dir}/uploads")),
            jwt_secret: var_or("JWT_SECRET", "change-me-in-production-please"),
            admin_username: var_or("ADMIN_USERNAME", "admin"),
            admin_password: var_or("ADMIN_PASSWORD", "admin123"),
            max_upload_mb: var_or("MAX_UPLOAD_MB", "100").parse().unwrap_or(100),
            static_dir: var_or("STATIC_DIR", "./static"),
        }
    }
}
