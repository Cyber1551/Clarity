use serde::{Deserialize, Serialize};

/// Represents a physical file on disk in user-visible directories.
///
/// Does not include media_files in the .objects directory. Multiple media_files can reference
/// the same media content via media_id (deduplication).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaFileRow {
    pub id: i64,
    pub media_id: i64,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
    pub size_bytes: i64,
    pub mtime: i64,
    /// Last time this file was seen during a scan
    pub last_seen_mtime: i64,
    pub is_reviewed: bool,
    pub original_file_name: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl MediaFileRow {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            media_id: row.get("media_id")?,
            rel_path: row.get("rel_path")?,
            dir_path: row.get("dir_path")?,
            file_name: row.get("file_name")?,
            ext: row.get("ext")?,
            size_bytes: row.get("size_bytes")?,
            mtime: row.get("mtime")?,
            last_seen_mtime: row.get("last_seen_mtime")?,
            is_reviewed: row.get::<_, i64>("is_reviewed")? != 0,
            original_file_name: row.get("original_file_name")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

#[derive(Debug)]
pub struct NewFileRecord<'a> {
    pub media_id: i64,
    pub rel_path: &'a str,
    pub dir_path: &'a String,
    pub file_name: &'a String,
    pub ext: &'a String,
    pub size_bytes: i64,
    pub mtime: i64,
    pub now: i64,
}

#[derive(Debug, Clone)]
pub struct UpsertFileResult {
    pub file_entry: MediaFileRow,
    pub is_new: bool,
    pub mtime_changed: bool,
}