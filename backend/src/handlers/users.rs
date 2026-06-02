use crate::auth::{hash_password, AdminUser};
use crate::error::{AppError, AppResult};
use crate::handlers::{validate_password, validate_username};
use crate::models::{CreateUser, ResetPassword, UserDto};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

pub async fn list_users(
    State(st): State<AppState>,
    _admin: AdminUser,
) -> AppResult<Json<Vec<UserDto>>> {
    let rows: Vec<(i64, String, bool, String, i64)> = sqlx::query_as(
        "SELECT u.id, u.username, u.is_admin, u.created_at, \
         (SELECT COUNT(*) FROM files f WHERE f.user_id = u.id) AS file_count \
         FROM users u ORDER BY u.id ASC",
    )
    .fetch_all(&st.pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|(id, username, is_admin, created_at, file_count)| UserDto {
            id,
            username,
            is_admin,
            created_at,
            file_count,
        })
        .collect();
    Ok(Json(users))
}

pub async fn create_user(
    State(st): State<AppState>,
    _admin: AdminUser,
    Json(body): Json<CreateUser>,
) -> AppResult<Json<Value>> {
    validate_username(&body.username)?;
    validate_password(&body.password)?;
    let username = body.username.trim().to_string();

    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&st.pool)
        .await?;
    if exists.is_some() {
        return Err(AppError::conflict("用户名已存在"));
    }

    let hash = hash_password(&body.password)?;
    let now = Utc::now().to_rfc3339();
    let rec: (i64,) = sqlx::query_as(
        "INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(&username)
    .bind(&hash)
    .bind(body.is_admin)
    .bind(&now)
    .fetch_one(&st.pool)
    .await?;

    Ok(Json(json!({ "id": rec.0, "username": username, "is_admin": body.is_admin })))
}

pub async fn delete_user(
    State(st): State<AppState>,
    admin: AdminUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    if id == admin.0.id {
        return Err(AppError::bad("不能删除当前登录的管理员账号"));
    }
    // Files (and their visits) are removed via ON DELETE CASCADE.
    // We also delete the physical files first.
    let stored: Vec<(String,)> = sqlx::query_as("SELECT stored_name FROM files WHERE user_id = ?")
        .bind(id)
        .fetch_all(&st.pool)
        .await?;
    for (name,) in &stored {
        let path = std::path::Path::new(&st.config.upload_dir).join(name);
        let _ = tokio::fs::remove_file(path).await;
    }

    let res = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&st.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(json!({ "ok": true })))
}

pub async fn reset_password(
    State(st): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i64>,
    Json(body): Json<ResetPassword>,
) -> AppResult<Json<Value>> {
    validate_password(&body.new_password)?;
    let hash = hash_password(&body.new_password)?;
    let res = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&hash)
        .bind(id)
        .execute(&st.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(json!({ "ok": true })))
}
