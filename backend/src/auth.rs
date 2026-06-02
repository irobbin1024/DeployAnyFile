use crate::error::AppError;
use crate::state::AppState;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("hash: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

pub fn create_token(secret: &str, id: i64, username: &str, is_admin: bool) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let claims = Claims {
        sub: id,
        username: username.to_string(),
        is_admin,
        exp,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("jwt: {e}")))
}

fn decode_token(secret: &str, token: &str) -> Option<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|d| d.claims)
}

fn token_from_parts(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Authenticated user (required).
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = token_from_parts(parts).ok_or(AppError::Unauthorized)?;
        let claims = decode_token(&state.config.jwt_secret, &token).ok_or(AppError::Unauthorized)?;
        Ok(AuthUser {
            id: claims.sub,
            username: claims.username,
            is_admin: claims.is_admin,
        })
    }
}

/// Admin-only user.
pub struct AdminUser(pub AuthUser);

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if !user.is_admin {
            return Err(AppError::Forbidden);
        }
        Ok(AdminUser(user))
    }
}

/// Optional authenticated user (for public pages that behave differently for owners).
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = match token_from_parts(parts) {
            Some(token) => decode_token(&state.config.jwt_secret, &token).map(|c| AuthUser {
                id: c.sub,
                username: c.username,
                is_admin: c.is_admin,
            }),
            None => None,
        };
        Ok(OptionalAuthUser(user))
    }
}
