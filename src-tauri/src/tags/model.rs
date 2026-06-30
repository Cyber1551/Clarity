use serde::{Deserialize, Serialize};

/// A user tag. `name` is the user-facing label; `slug` is the filesystem-safe form used to name the `Library/By Tag/<slug>` projection folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagRow {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TagRow {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            slug: row.get("slug")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// Lowercase, filesystem-safe, hyphen-separated slug; falls back to `"untitled"` when the input has no usable characters.
pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = false;

    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.to_string()
    }
}
