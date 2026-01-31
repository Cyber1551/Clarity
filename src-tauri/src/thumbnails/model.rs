use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThumbnailRow {
    pub content_hash: String,
    pub thumbnail_blob: Vec<u8>,
    pub mimetype: String,
    pub width: i32,
    pub height: i32,
}

impl ThumbnailRow {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            content_hash: row.get("content_hash")?,
            thumbnail_blob: row.get("thumbnail_blob")?,
            mimetype: row.get("mimetype")?,
            width: row.get("width")?,
            height: row.get("height")?,
        })
    }
}