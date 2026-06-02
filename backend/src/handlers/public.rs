use crate::auth::OptionalAuthUser;
use crate::error::{AppError, AppResult};
use crate::handlers::client_ip;
use crate::models::FileRow;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};
use std::net::SocketAddr;

async fn fetch_by_slug(st: &AppState, slug: &str) -> AppResult<FileRow> {
    let row: Option<FileRow> = sqlx::query_as("SELECT * FROM files WHERE slug = ?")
        .bind(slug)
        .fetch_optional(&st.pool)
        .await?;
    row.ok_or(AppError::NotFound)
}

/// Public metadata for the preview page. Records a visit unless the viewer is the owner.
pub async fn public_meta(
    State(st): State<AppState>,
    Path(slug): Path<String>,
    OptionalAuthUser(viewer): OptionalAuthUser,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
) -> AppResult<Json<Value>> {
    let file = fetch_by_slug(&st, &slug).await?;
    let is_owner = viewer.as_ref().map(|u| u.id) == Some(file.user_id);

    if !file.is_shared && !is_owner {
        return Err(AppError::Forbidden);
    }

    // Record a visit only for non-owner viewers.
    if !is_owner {
        let ip = client_ip(&headers, Some(peer));
        let ua = headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.chars().take(300).collect::<String>());
        let _ = sqlx::query("INSERT INTO visits (file_id, ip, user_agent, visited_at) VALUES (?, ?, ?, ?)")
            .bind(file.id)
            .bind(&ip)
            .bind(ua)
            .bind(Utc::now().to_rfc3339())
            .execute(&st.pool)
            .await;
    }

    let owner: Option<(String,)> = sqlx::query_as("SELECT username FROM users WHERE id = ?")
        .bind(file.user_id)
        .fetch_optional(&st.pool)
        .await?;

    Ok(Json(json!({
        "id": file.id,
        "slug": file.slug,
        "original_name": file.original_name,
        "mime_type": file.mime_type,
        "category": file.category,
        "size": file.size,
        "is_shared": file.is_shared,
        "created_at": file.created_at,
        "owner": owner.map(|o| o.0),
        "is_owner": is_owner,
        "raw_url": format!("/raw/{}", file.slug),
    })))
}

/// Serve the raw file bytes with the correct content type (inline).
pub async fn raw(
    State(st): State<AppState>,
    Path(slug): Path<String>,
    OptionalAuthUser(viewer): OptionalAuthUser,
) -> Response {
    let file = match fetch_by_slug(&st, &slug).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "Not Found").into_response(),
    };
    let is_owner = viewer.as_ref().map(|u| u.id) == Some(file.user_id);
    if !file.is_shared && !is_owner {
        return (StatusCode::FORBIDDEN, "该分享已关闭").into_response();
    }

    let path = std::path::Path::new(&st.config.upload_dir).join(&file.stored_name);
    let data = match tokio::fs::read(&path).await {
        Ok(d) => d,
        Err(_) => return (StatusCode::NOT_FOUND, "文件已丢失").into_response(),
    };

    let disposition = format!(
        "inline; filename*=UTF-8''{}",
        urlencode(&file.original_name)
    );
    Response::builder()
        .header(header::CONTENT_TYPE, file.mime_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(data))
        .unwrap()
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
