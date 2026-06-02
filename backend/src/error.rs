use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("internal error")]
    Internal(String),
}

impl AppError {
    pub fn bad(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        AppError::Conflict(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "未授权或登录已过期".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "没有权限".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "资源不存在".to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::Internal(m) => {
                tracing::error!("internal error: {m}");
                (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误".to_string())
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(format!("db: {e}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(format!("io: {e}"))
    }
}

impl From<axum::extract::multipart::MultipartError> for AppError {
    fn from(e: axum::extract::multipart::MultipartError) -> Self {
        AppError::BadRequest(format!("上传解析失败: {e}"))
    }
}

pub type AppResult<T> = Result<T, AppError>;
