use crate::auth::{create_token, hash_password, verify_password, AuthUser};
use crate::error::{AppError, AppResult};
use crate::handlers::{validate_password, validate_username};
use crate::models::{ChangePassword, Credentials, User};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

pub async fn register(
    State(st): State<AppState>,
    Json(body): Json<Credentials>,
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
        "INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, 0, ?) RETURNING id",
    )
    .bind(&username)
    .bind(&hash)
    .bind(&now)
    .fetch_one(&st.pool)
    .await?;

    let token = create_token(&st.config.jwt_secret, rec.0, &username, false)?;
    Ok(Json(json!({
        "token": token,
        "user": { "id": rec.0, "username": username, "is_admin": false }
    })))
}

pub async fn login(
    State(st): State<AppState>,
    Json(body): Json<Credentials>,
) -> AppResult<Json<Value>> {
    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(body.username.trim())
        .fetch_optional(&st.pool)
        .await?;

    let user = user.ok_or_else(|| AppError::bad("用户名或密码错误"))?;
    if !verify_password(&body.password, &user.password_hash) {
        return Err(AppError::bad("用户名或密码错误"));
    }

    let token = create_token(&st.config.jwt_secret, user.id, &user.username, user.is_admin)?;
    Ok(Json(json!({
        "token": token,
        "user": { "id": user.id, "username": user.username, "is_admin": user.is_admin }
    })))
}

pub async fn me(State(st): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let row: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_optional(&st.pool)
        .await?;
    let row = row.ok_or(AppError::Unauthorized)?;
    Ok(Json(json!({
        "id": row.id, "username": row.username, "is_admin": row.is_admin, "created_at": row.created_at
    })))
}

pub async fn change_password(
    State(st): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChangePassword>,
) -> AppResult<Json<Value>> {
    validate_password(&body.new_password)?;
    let row: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_one(&st.pool)
        .await?;
    if !verify_password(&body.old_password, &row.password_hash) {
        return Err(AppError::bad("原密码错误"));
    }
    let hash = hash_password(&body.new_password)?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&hash)
        .bind(user.id)
        .execute(&st.pool)
        .await?;
    Ok(Json(json!({ "ok": true })))
}
