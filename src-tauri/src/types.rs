use serde::{Serialize, Deserialize};

/// Media Item Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItemResponse {
    pub id: Option<i64>,
    pub path: String,
    pub file_name: String,
    pub file_size: i32,
    pub file_extension: String,
    pub media_type: String,
    pub video_length: Option<f64>,
    pub thumbnail_base64: String,
    pub created_at: i64,
    pub updated_at: i64,
}