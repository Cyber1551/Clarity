use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: i64,
    pub media_id: Option<i64>,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
    pub size_bytes: i64,
    pub mtime: i64,
    pub last_scan_mtime: i64,
    pub is_reviewed: bool,
    pub created_at: i64,
    pub updated_at: i64,
}