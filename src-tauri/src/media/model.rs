use serde::{Deserialize, Serialize};
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use strum::{Display, EnumString};
use crate::core::constants::{VALID_IMAGE_EXTENSIONS, VALID_VIDEO_EXTENSIONS};
use crate::jobs::JobStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")] // Serializes as "image" instead of "Image"
#[strum(serialize_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Unknown
}

impl MediaType {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaEntry {
    pub id: i64,
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