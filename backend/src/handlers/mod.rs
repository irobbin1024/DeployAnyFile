pub mod auth;
pub mod files;
pub mod public;
pub mod users;

use crate::error::AppError;
use axum::http::HeaderMap;
use std::net::SocketAddr;

/// Extract the best-effort client IP from proxy headers or the socket address.
pub fn client_ip(headers: &HeaderMap, peer: Option<SocketAddr>) -> String {
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            let ip = first.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }
    if let Some(real) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        if !real.trim().is_empty() {
            return real.trim().to_string();
        }
    }
    peer.map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".into())
}

pub fn validate_username(name: &str) -> Result<(), AppError> {
    let n = name.trim();
    if n.len() < 3 || n.len() > 32 {
        return Err(AppError::bad("用户名长度需在 3-32 之间"));
    }
    if !n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::bad("用户名只能包含字母、数字、'_' 和 '-'"));
    }
    Ok(())
}

pub fn validate_password(pw: &str) -> Result<(), AppError> {
    if pw.len() < 6 {
        return Err(AppError::bad("密码长度至少 6 位"));
    }
    Ok(())
}
