use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub created_at: String,
    pub file_count: i64,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct FileRow {
    pub id: i64,
    pub user_id: i64,
    pub slug: String,
    pub original_name: String,
    pub stored_name: String,
    pub mime_type: String,
    pub category: String,
    pub size: i64,
    pub is_shared: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FileDto {
    pub id: i64,
    pub slug: String,
    pub original_name: String,
    pub mime_type: String,
    pub category: String,
    pub size: i64,
    pub is_shared: bool,
    pub created_at: String,
    pub view_count: i64,
}

impl FileRow {
    pub fn into_dto(self, view_count: i64) -> FileDto {
        FileDto {
            id: self.id,
            slug: self.slug,
            original_name: self.original_name,
            mime_type: self.mime_type,
            category: self.category,
            size: self.size,
            is_shared: self.is_shared,
            created_at: self.created_at,
            view_count,
        }
    }
}

#[derive(Debug, FromRow, Serialize)]
pub struct VisitRow {
    pub ip: String,
    pub user_agent: Option<String>,
    pub visited_at: String,
}

// ---- Request payloads ----

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct ResetPassword {
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct IdList {
    pub ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSlug {
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct SetShare {
    pub is_shared: bool,
}
