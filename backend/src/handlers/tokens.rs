use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::models::{ApiTokenDto, CreateTokenReq};
use crate::state::AppState;
use crate::util::{gen_api_token, sha256_hex};
use axum::extract::{Path, State};
use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

pub async fn list_tokens(
    State(st): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<ApiTokenDto>>> {
    let rows: Vec<ApiTokenDto> = sqlx::query_as(
        "SELECT id, name, token_prefix, created_at, last_used_at \
         FROM api_tokens WHERE user_id = ? ORDER BY id DESC",
    )
    .bind(user.id)
    .fetch_all(&st.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create_token(
    State(st): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateTokenReq>,
) -> AppResult<Json<Value>> {
    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > 50 {
        return Err(AppError::bad("令牌名称需在 1-50 个字符之间"));
    }

    let token = gen_api_token();
    let hash = sha256_hex(&token);
    let prefix = format!("{}…", &token[..12]); // e.g. "daf_AbCdEf…"
    let now = Utc::now().to_rfc3339();

    let rec: (i64,) = sqlx::query_as(
        "INSERT INTO api_tokens (user_id, name, token_hash, token_prefix, created_at) \
         VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(user.id)
    .bind(&name)
    .bind(&hash)
    .bind(&prefix)
    .bind(&now)
    .fetch_one(&st.pool)
    .await?;

    // The full token is returned ONCE here and never stored in plaintext.
    Ok(Json(json!({
        "id": rec.0,
        "name": name,
        "token": token,
        "token_prefix": prefix,
        "created_at": now,
    })))
}

pub async fn delete_token(
    State(st): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let res = sqlx::query("DELETE FROM api_tokens WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.id)
        .execute(&st.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(json!({ "ok": true })))
}
