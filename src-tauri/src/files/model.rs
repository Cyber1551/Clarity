use serde::{Deserialize, Serialize};

/// Represents a physical file on disk in user-visible directories.
///
/// Does not include files in the .objects directory. Multiple files can reference
/// the same media content via media_id (deduplication).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: i64,
    /// References the media table if file has been hashed
    pub media_id: Option<i64>,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
    pub size_bytes: i64,
    /// Modification time from filesystem
    pub mtime: i64,
    /// Last time this file was seen during a scan
    pub last_seen_mtime: i64,
    pub is_reviewed: bool,
    pub created_at: i64,
    pub updated_at: i64,
}