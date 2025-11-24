use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use crate::jobs::JobStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // Serializes as "image" instead of "Image"
pub enum MediaType {
    Image,
    Video,
    Unknown
}

// Allows: my_enum.to_string() -> "image"
impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MediaType::Image => "image",
            MediaType::Video => "video",
            MediaType::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

// Allows: "image".parse::<MediaType>()
impl FromStr for MediaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "image" => Ok(MediaType::Image),
            "video" => Ok(MediaType::Video),
            _ => Ok(MediaType::Unknown)
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
    pub content_hash: Option<String>,
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