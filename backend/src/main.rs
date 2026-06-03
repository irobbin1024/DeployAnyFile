mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod state;
mod util;

use axum::extract::DefaultBodyLimit;
use axum::http::header::CACHE_CONTROL;
use axum::http::HeaderValue;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use config::Config;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=warn".into()),
        )
        .init();

    let config = Config::from_env();
    std::fs::create_dir_all(&config.upload_dir).ok();

    let pool = db::init_pool(&config).await?;
    db::bootstrap_admin(&pool, &config).await?;

    let max_body = config.max_upload_mb * 1024 * 1024 + 1024 * 1024;
    let static_dir = config.static_dir.clone();
    let bind_addr = config.bind_addr.clone();

    let state = AppState {
        pool,
        config: Arc::new(config),
    };

    let api = Router::new()
        // auth
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/me", get(handlers::auth::me))
        .route("/auth/change-password", post(handlers::auth::change_password))
        // admin user management
        .route("/users", get(handlers::users::list_users).post(handlers::users::create_user))
        .route("/users/:id", delete(handlers::users::delete_user))
        .route("/users/:id/reset-password", post(handlers::users::reset_password))
        // files
        .route(
            "/files",
            get(handlers::files::list_files).delete(handlers::files::delete_files),
        )
        .route(
            "/files/upload",
            post(handlers::files::upload).layer(DefaultBodyLimit::max(max_body)),
        )
        .route("/files/share", post(handlers::files::set_share_bulk))
        .route("/files/:id/share", patch(handlers::files::set_share_one))
        .route("/files/:id/slug", patch(handlers::files::update_slug))
        .route("/files/:id/stats", get(handlers::files::file_stats))
        .route("/stats", get(handlers::files::site_stats))
        // public metadata
        .route("/public/:slug", get(handlers::public::public_meta))
        // never cache API responses (avoids stale lists behind proxies/browsers)
        .layer(SetResponseHeaderLayer::overriding(
            CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ));

    let serve_dir = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(format!("{static_dir}/index.html")));

    let app = Router::new()
        .nest("/api", api)
        .route("/raw/:slug", get(handlers::public::raw))
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = bind_addr.parse()?;
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
