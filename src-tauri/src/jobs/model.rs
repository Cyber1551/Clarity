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

/// A background job row from the jobs table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRow {
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

impl JobRow {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            job_type: row.get("job_type")?,
            media_id: row.get("media_id")?,
            file_id: row.get("file_id")?,
            rel_path: row.get("rel_path")?,
            queued_mtime: row.get("queued_mtime")?,
            priority: row.get("priority")?,
            status: row.get("status")?,
            attempts: row.get("attempts")?,
            last_error: row.get("last_error")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

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

/// Parameters for enqueueing a background job.
pub struct EnqueueJobRequest {
    pub file_id: i64,
    pub media_id: Option<i64>,
    pub rel_path: String,
    /// Modification time at queue time, used to detect stale jobs
    pub mtime: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_is_pending_or_error() {
        assert!(JobStatus::Pending.is_pending_or_error());
        assert!(JobStatus::Error.is_pending_or_error());
        assert!(!JobStatus::Done.is_pending_or_error());
        assert!(!JobStatus::Processing.is_pending_or_error());
    }

    #[test]
    fn test_job_type_display() {
        assert_eq!(JobType::Metadata.to_string(), "metadata");
        assert_eq!(JobType::Thumbnail.to_string(), "thumbnail");
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Processing.to_string(), "processing");
        assert_eq!(JobStatus::Done.to_string(), "done");
        assert_eq!(JobStatus::Error.to_string(), "error");
    }

    #[test]
    fn test_job_entry_require_file_id() {
        let job = JobRow {
            id: 1,
            job_type: JobType::Metadata,
            media_id: None,
            file_id: Some(42),
            rel_path: None,
            queued_mtime: None,
            priority: 0,
            status: JobStatus::Pending,
            attempts: 0,
            last_error: None,
            created_at: 0,
            updated_at: 0,
        };

        assert_eq!(job.require_file_id().unwrap(), 42);

        let job_without_file_id = JobRow {
            file_id: None,
            ..job
        };

        assert!(job_without_file_id.require_file_id().is_err());
    }

    #[test]
    fn test_job_entry_require_media_id() {
        let job = JobRow {
            id: 1,
            job_type: JobType::Metadata,
            media_id: Some(99),
            file_id: None,
            rel_path: None,
            queued_mtime: None,
            priority: 0,
            status: JobStatus::Pending,
            attempts: 0,
            last_error: None,
            created_at: 0,
            updated_at: 0,
        };

        assert_eq!(job.require_media_id().unwrap(), 99);

        let job_without_media_id = JobRow {
            media_id: None,
            ..job
        };

        assert!(job_without_media_id.require_media_id().is_err());
    }
}