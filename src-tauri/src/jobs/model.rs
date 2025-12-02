use serde::{Deserialize, Serialize};
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use strum::{Display, EnumString};
use crate::core::error::{AppError, AppResult};

/// Type of background job to execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum JobType {
    /// Compute blake3 hash and deduplicate to .objects directory
    Hash,
    /// Extract media metadata (dimensions, duration)
    Metadata,
    /// Generate thumbnail
    Thumbnail,
}

impl FromSql for JobType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        s.parse::<JobType>().map_err(|e| FromSqlError::Other(Box::new(e)) )
    }
}

impl ToSql for JobType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

/// Status of a background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum JobStatus {
    Done,
    Pending,
    Processing,
    Error
}

impl JobStatus {
    pub fn is_pending_or_error(&self) -> bool {
        matches!(self, JobStatus::Pending | JobStatus::Error)
    }
}

impl FromSql for JobStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = value.as_str()?;
        Ok(s.parse::<JobStatus>().unwrap_or(JobStatus::Error))
    }
}

impl ToSql for JobStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

/// A background job entry from the jobs table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobEntry {
    pub id: i64,
    pub job_type: JobType,
    pub media_id: Option<i64>,
    pub file_id: Option<i64>,
    pub rel_path: Option<String>,
    /// Modification time when job was queued, used to detect stale jobs
    pub queued_mtime: Option<i64>,
    pub priority: i32,
    pub status: JobStatus,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl JobEntry {
    /// Returns the file_id or an error if it's None.
    pub fn require_file_id(&self) -> AppResult<i64> {
        self.file_id.ok_or_else(|| {
            AppError::Other(format!("Job {} ({:?}) missing file_id", self.id, self.job_type))
        })
    }

    /// Returns the media_id or an error if it's None.
    pub fn require_media_id(&self) -> AppResult<i64> {
        self.media_id.ok_or_else(|| {
            AppError::Other(format!("Job {} ({:?}) missing media_id", self.id, self.job_type))
        })
    }
}