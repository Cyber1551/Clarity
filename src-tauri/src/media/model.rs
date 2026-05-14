use serde::{Deserialize, Serialize};
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use strum::{Display, EnumString};
use crate::core::constants::{VALID_IMAGE_EXTENSIONS, VALID_VIDEO_EXTENSIONS};
use crate::jobs::JobStatus;

/// Type of media content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Unknown
}

impl MediaType {
    /// Determines media type from a file extension.
    pub fn from_extension(ext: &str) -> Self {
        let ext_lower = ext.to_lowercase();
        if VALID_IMAGE_EXTENSIONS.contains(&ext_lower.as_str()) {
            MediaType::Image
        } else if VALID_VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
            MediaType::Video
        } else {
            MediaType::Unknown
        }
    }
}

impl FromSql for MediaType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        Ok(s.parse::<MediaType>().unwrap_or(MediaType::Unknown))
    }
}

impl ToSql for MediaType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

/// Represents unique media content identified by content_hash.
///
/// Multiple media_files can reference the same media row (deduplication).
/// Tracks processing status for hashing, metadata extraction, and thumbnail generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaRow {
    pub id: i64,
    /// Blake3 hash of the file content
    pub content_hash: String,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub quality_rating: i32,
    pub favorite_rating: i32,
    pub loved: bool,
    pub hash_status: JobStatus,
    pub metadata_status: JobStatus,
    pub thumbnail_status: JobStatus,
    pub reviewed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl MediaRow {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            content_hash: row.get("content_hash")?,
            media_type: row.get("media_type")?,
            width: row.get("width")?,
            height: row.get("height")?,
            duration_ms: row.get("duration_ms")?,
            quality_rating: row.get("quality_rating")?,
            favorite_rating: row.get("favorite_rating")?,
            loved: row.get::<_, i64>("loved")? != 0,
            hash_status: row.get("hash_status")?,
            metadata_status: row.get("metadata_status")?,
            thumbnail_status: row.get("thumbnail_status")?,
            reviewed_at: row.get("reviewed_at")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// A domain object representing a piece of media along with its primary filesystem path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub media: MediaRow,
    /// The representative path for this media item (from the media_files table)
    pub rel_path: Option<String>,
    pub dir_path: Option<String>,
    pub file_name: Option<String>,
    pub ext: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_image_extension() {
        assert_eq!(MediaType::from_extension("jpg"), MediaType::Image);
        assert_eq!(MediaType::from_extension("JPG"), MediaType::Image);
        assert_eq!(MediaType::from_extension("png"), MediaType::Image);
        assert_eq!(MediaType::from_extension("webp"), MediaType::Image);
    }

    #[test]
    fn test_media_type_from_video_extension() {
        assert_eq!(MediaType::from_extension("mp4"), MediaType::Video);
        assert_eq!(MediaType::from_extension("MP4"), MediaType::Video);
        assert_eq!(MediaType::from_extension("mov"), MediaType::Video);
        assert_eq!(MediaType::from_extension("mkv"), MediaType::Video);
    }

    #[test]
    fn test_media_type_from_unknown_extension() {
        assert_eq!(MediaType::from_extension("txt"), MediaType::Unknown);
        assert_eq!(MediaType::from_extension("pdf"), MediaType::Unknown);
        assert_eq!(MediaType::from_extension(""), MediaType::Unknown);
    }
}