use crate::auth::{ApiUser, AuthUser};
use crate::error::{AppError, AppResult};
use crate::models::{FileDto, FileRow, IdList, SetShare, UpdateSlug, VisitRow};
use crate::state::AppState;
use crate::util::{categorize, random_slug, resolve_mime, validate_slug};
use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{QueryBuilder, Sqlite};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct FileListRow {
    id: i64,
    slug: String,
    original_name: String,
    mime_type: String,
    category: String,
    size: i64,
    is_shared: bool,
    created_at: String,
    view_count: i64,
}

impl From<FileListRow> for FileDto {
    fn from(r: FileListRow) -> Self {
        FileDto {
            id: r.id,
            slug: r.slug,
            original_name: r.original_name,
            mime_type: r.mime_type,
            category: r.category,
            size: r.size,
            is_shared: r.is_shared,
            created_at: r.created_at,
            view_count: r.view_count,
        }
    }
}

// ---------- Upload ----------

pub async fn upload(
    State(st): State<AppState>,
    user: ApiUser,
    mut mp: Multipart,
) -> AppResult<Json<FileDto>> {
    let mut custom_slug: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;
    let mut original_name = String::from("file");
    let mut content_type = String::new();

    while let Some(field) = mp.next_field().await? {
        match field.name() {
            Some("slug") => {
                let v = field.text().await?;
                let v = v.trim().to_string();
                if !v.is_empty() {
                    custom_slug = Some(v);
                }
            }
            Some("file") => {
                if let Some(fname) = field.file_name() {
                    original_name = fname.to_string();
                }
                content_type = field.content_type().map(|s| s.to_string()).unwrap_or_default();
                bytes = Some(field.bytes().await?.to_vec());
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let data = bytes.ok_or_else(|| AppError::bad("缺少上传文件"))?;
    if data.is_empty() {
        return Err(AppError::bad("文件内容为空"));
    }
    let max = st.config.max_upload_mb * 1024 * 1024;
    if data.len() > max {
        return Err(AppError::bad(format!(
            "文件过大，最大允许 {} MB",
            st.config.max_upload_mb
        )));
    }

    let mime = resolve_mime(&content_type, &original_name);
    let category = categorize(&mime, &original_name);

    // Determine slug.
    let slug = match custom_slug {
        Some(s) => {
            validate_slug(&s)?;
            let taken: Option<(i64,)> = sqlx::query_as("SELECT id FROM files WHERE slug = ?")
                .bind(&s)
                .fetch_optional(&st.pool)
                .await?;
            if taken.is_some() {
                return Err(AppError::conflict("该链接地址已被占用，请更换"));
            }
            s
        }
        None => unique_random_slug(&st).await?,
    };

    // Store on disk with a uuid name, preserving extension.
    let ext = std::path::Path::new(&original_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let stored_name = format!("{}{}", Uuid::new_v4(), ext);
    let path = std::path::Path::new(&st.config.upload_dir).join(&stored_name);
    tokio::fs::create_dir_all(&st.config.upload_dir).await?;
    tokio::fs::write(&path, &data).await?;

    let now = Utc::now().to_rfc3339();
    let size = data.len() as i64;
    let rec: (i64,) = sqlx::query_as(
        "INSERT INTO files (user_id, slug, original_name, stored_name, mime_type, category, size, is_shared, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?) RETURNING id",
    )
    .bind(user.id)
    .bind(&slug)
    .bind(&original_name)
    .bind(&stored_name)
    .bind(&mime)
    .bind(&category)
    .bind(size)
    .bind(&now)
    .fetch_one(&st.pool)
    .await?;

    Ok(Json(FileDto {
        id: rec.0,
        slug,
        original_name,
        mime_type: mime,
        category,
        size,
        is_shared: true,
        created_at: now,
        view_count: 0,
    }))
}

async fn unique_random_slug(st: &AppState) -> AppResult<String> {
    for len in [6usize, 6, 7, 7, 8, 8, 9, 10] {
        let candidate = random_slug(len);
        let taken: Option<(i64,)> = sqlx::query_as("SELECT id FROM files WHERE slug = ?")
            .bind(&candidate)
            .fetch_optional(&st.pool)
            .await?;
        if taken.is_none() {
            return Ok(candidate);
        }
    }
    Err(AppError::Internal("无法生成唯一链接，请重试".into()))
}

// ---------- List ----------

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    12
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

pub async fn list_files(
    State(st): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    let page = q.page.max(1);
    let page_size = q.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let category = q
        .category
        .filter(|c| !c.is_empty() && c.as_str() != "all");
    let search = q.search.filter(|s| !s.trim().is_empty());

    // Count.
    let mut cb: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT COUNT(*) FROM files WHERE user_id = ");
    cb.push_bind(user.id);
    if let Some(cat) = &category {
        cb.push(" AND category = ").push_bind(cat.clone());
    }
    if let Some(s) = &search {
        cb.push(" AND original_name LIKE ")
            .push_bind(format!("%{}%", s.trim()));
    }
    let total: i64 = cb.build_query_scalar().fetch_one(&st.pool).await?;

    // Page of rows.
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT id, slug, original_name, mime_type, category, size, is_shared, created_at, \
         (SELECT COUNT(*) FROM visits v WHERE v.file_id = files.id) AS view_count \
         FROM files WHERE user_id = ",
    );
    qb.push_bind(user.id);
    if let Some(cat) = &category {
        qb.push(" AND category = ").push_bind(cat.clone());
    }
    if let Some(s) = &search {
        qb.push(" AND original_name LIKE ")
            .push_bind(format!("%{}%", s.trim()));
    }
    qb.push(" ORDER BY id DESC LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows: Vec<FileListRow> = qb.build_query_as().fetch_all(&st.pool).await?;
    let items: Vec<FileDto> = rows.into_iter().map(FileDto::from).collect();

    Ok(Json(json!({
        "items": items,
        "total": total,
        "page": page,
        "page_size": page_size,
    })))
}

// ---------- Mutations ----------

async fn owned_file(st: &AppState, id: i64, user_id: i64) -> AppResult<FileRow> {
    let row: Option<FileRow> = sqlx::query_as("SELECT * FROM files WHERE id = ?")
        .bind(id)
        .fetch_optional(&st.pool)
        .await?;
    let row = row.ok_or(AppError::NotFound)?;
    if row.user_id != user_id {
        return Err(AppError::Forbidden);
    }
    Ok(row)
}

pub async fn delete_files(
    State(st): State<AppState>,
    user: AuthUser,
    Json(body): Json<IdList>,
) -> AppResult<Json<Value>> {
    if body.ids.is_empty() {
        return Err(AppError::bad("未选择任何文件"));
    }
    let mut deleted = 0u64;
    for id in body.ids {
        let row = match owned_file(&st, id, user.id).await {
            Ok(r) => r,
            Err(AppError::NotFound) | Err(AppError::Forbidden) => continue,
            Err(e) => return Err(e),
        };
        let path = std::path::Path::new(&st.config.upload_dir).join(&row.stored_name);
        let _ = tokio::fs::remove_file(path).await;
        let res = sqlx::query("DELETE FROM files WHERE id = ?")
            .bind(id)
            .execute(&st.pool)
            .await?;
        deleted += res.rows_affected();
    }
    Ok(Json(json!({ "deleted": deleted })))
}

#[derive(Deserialize)]
pub struct UnshareBody {
    pub ids: Vec<i64>,
    #[serde(default)]
    pub is_shared: bool,
}

pub async fn set_share_bulk(
    State(st): State<AppState>,
    user: AuthUser,
    Json(body): Json<UnshareBody>,
) -> AppResult<Json<Value>> {
    if body.ids.is_empty() {
        return Err(AppError::bad("未选择任何文件"));
    }
    let mut changed = 0u64;
    for id in body.ids {
        match owned_file(&st, id, user.id).await {
            Ok(_) => {}
            Err(AppError::NotFound) | Err(AppError::Forbidden) => continue,
            Err(e) => return Err(e),
        };
        let res = sqlx::query("UPDATE files SET is_shared = ? WHERE id = ?")
            .bind(body.is_shared)
            .bind(id)
            .execute(&st.pool)
            .await?;
        changed += res.rows_affected();
    }
    Ok(Json(json!({ "changed": changed })))
}

pub async fn set_share_one(
    State(st): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<SetShare>,
) -> AppResult<Json<Value>> {
    owned_file(&st, id, user.id).await?;
    sqlx::query("UPDATE files SET is_shared = ? WHERE id = ?")
        .bind(body.is_shared)
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(Json(json!({ "ok": true, "is_shared": body.is_shared })))
}

pub async fn update_slug(
    State(st): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<UpdateSlug>,
) -> AppResult<Json<Value>> {
    let new_slug = body.slug.trim().to_string();
    validate_slug(&new_slug)?;
    owned_file(&st, id, user.id).await?;

    let taken: Option<(i64,)> = sqlx::query_as("SELECT id FROM files WHERE slug = ? AND id != ?")
        .bind(&new_slug)
        .bind(id)
        .fetch_optional(&st.pool)
        .await?;
    if taken.is_some() {
        return Err(AppError::conflict("该链接地址已被占用，请更换"));
    }

    sqlx::query("UPDATE files SET slug = ? WHERE id = ?")
        .bind(&new_slug)
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(Json(json!({ "ok": true, "slug": new_slug })))
}

pub async fn site_stats(
    State(st): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<Value>> {
    let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(&st.pool)
        .await?;
    let shared_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE is_shared = 1")
        .fetch_one(&st.pool)
        .await?;
    let total_size: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(size), 0) FROM files")
        .fetch_one(&st.pool)
        .await?;
    let total_views: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM visits")
        .fetch_one(&st.pool)
        .await?;
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&st.pool)
        .await?;

    Ok(Json(json!({
        "total_files": total_files,
        "shared_files": shared_files,
        "total_size": total_size,
        "total_views": total_views,
        "total_users": total_users,
    })))
}

pub async fn file_stats(
    State(st): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let row = owned_file(&st, id, user.id).await?;

    let total_views: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM visits WHERE file_id = ?")
        .bind(id)
        .fetch_one(&st.pool)
        .await?;
    let unique_ips: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT ip) FROM visits WHERE file_id = ?")
            .bind(id)
            .fetch_one(&st.pool)
            .await?;
    let visits: Vec<VisitRow> = sqlx::query_as(
        "SELECT ip, user_agent, visited_at FROM visits WHERE file_id = ? ORDER BY id DESC LIMIT 500",
    )
    .bind(id)
    .fetch_all(&st.pool)
    .await?;

    Ok(Json(json!({
        "slug": row.slug,
        "original_name": row.original_name,
        "is_shared": row.is_shared,
        "share_created_at": row.created_at,
        "total_views": total_views,
        "unique_ips": unique_ips,
        "visits": visits,
    })))
}
