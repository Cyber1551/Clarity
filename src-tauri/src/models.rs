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
    pub media_id: i64,
    pub size: i32,
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
pub struct MediaMetadata {
    pub id: i64,           // Database ID of the media item
    pub duration: i32,
    pub thumbnail_base64: String,
    pub thumbnail_size: i32
}



/// Media item response returned to the frontend
#[derive(Serialize, Deserialize)]
pub struct MediaItemResponse {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub media_type: String,
    pub length: Option<i64>,
    pub thumbnail_base64: Option<String>,
    pub tags: Vec<String>,
    pub bookmarks: Vec<Bookmark>,
}
