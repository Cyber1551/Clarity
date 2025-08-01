use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: i64,
    pub path: PathBuf,
    pub file_name: String,
    pub file_size: u64,
    pub file_extension: String,
    pub media_type: String,
    pub video_length: Option<f64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thumbnail {
    pub media_id: i64,
    pub data: Vec<u8>,
    pub mime_type: String,
}