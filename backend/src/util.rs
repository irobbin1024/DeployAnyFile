use crate::error::AppError;
use rand::Rng;

const SLUG_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// Generate a random base62 slug of the given length.
pub fn random_slug(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| SLUG_CHARS[rng.gen_range(0..SLUG_CHARS.len())] as char)
        .collect()
}

/// Validate a user-supplied custom slug.
/// Allowed: letters, digits, '-', '_'; length 1..=64; not a reserved word.
pub fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.is_empty() || slug.len() > 64 {
        return Err(AppError::bad("链接地址长度需在 1-64 之间"));
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::bad("链接地址只能包含字母、数字、'-' 和 '_'"));
    }
    const RESERVED: [&str; 6] = ["api", "raw", "s", "p", "login", "register"];
    if RESERVED.contains(&slug.to_lowercase().as_str()) {
        return Err(AppError::bad("该链接地址为系统保留字，请更换"));
    }
    Ok(())
}

/// Classify a file into a coarse category for filtering / rendering.
pub fn categorize(mime: &str, filename: &str) -> String {
    let lower = filename.to_lowercase();
    if mime.starts_with("image/") {
        return "image".into();
    }
    if mime.starts_with("video/") {
        return "video".into();
    }
    if mime.starts_with("audio/") {
        return "audio".into();
    }
    if mime == "text/html" || lower.ends_with(".html") || lower.ends_with(".htm") {
        return "html".into();
    }
    if mime == "text/markdown" || lower.ends_with(".md") || lower.ends_with(".markdown") {
        return "markdown".into();
    }
    if mime.starts_with("text/")
        || lower.ends_with(".txt")
        || lower.ends_with(".json")
        || lower.ends_with(".csv")
        || lower.ends_with(".log")
        || mime == "application/json"
    {
        return "text".into();
    }
    "other".into()
}

/// Resolve a reasonable mime type from the supplied content type or the filename.
pub fn resolve_mime(content_type: &str, filename: &str) -> String {
    if !content_type.is_empty() && content_type != "application/octet-stream" {
        return content_type.to_string();
    }
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}
