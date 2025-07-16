use serde::{Serialize, Deserialize};

/// Media item stored in the database
#[derive(Debug, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Option<i64>,
    pub path: String,
    pub title: String,
    pub media_type: String,
    pub length: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Thumbnail stored in the database
#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub id: Option<i64>,
    pub media_id: i64,
    pub data: Vec<u8>,
    pub mime_type: String,
}

/// Tag stored in the database
#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}

/// Bookmark stored in the database
#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Option<i64>,
    pub media_id: i64,
    pub description: String,
    pub timestamp: i64,
}

/// Video metadata returned to the frontend
#[derive(Serialize, Deserialize)]
pub struct VideoMetadata {
    pub id: i64,           // Database ID of the media item
    pub thumbnail_id: i64, // Database ID of the thumbnail
    pub duration: u32,     // Duration in seconds
}

/// Media item response returned to the frontend
#[derive(Serialize, Deserialize)]
pub struct MediaItemResponse {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub media_type: String,
    pub length: Option<i64>,
    pub thumbnail_id: Option<i64>, // ID of the thumbnail in the thumbnails table
    pub tags: Vec<String>,
    pub bookmarks: Vec<Bookmark>,
}
