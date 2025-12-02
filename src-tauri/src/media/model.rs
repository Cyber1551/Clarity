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
/// Multiple files can reference the same media entry (deduplication).
/// Tracks processing status for hashing, metadata extraction, and thumbnail generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaEntry {
    pub id: i64,
    /// Blake3 hash of the file content
    pub content_hash: String,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub hash_status: JobStatus,
    pub metadata_status: JobStatus,
    pub thumbnail_status: JobStatus,
    pub created_at: i64,
    pub updated_at: i64,
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