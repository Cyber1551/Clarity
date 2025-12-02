use serde::{Deserialize, Serialize};
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use strum::{Display, EnumString};
use crate::core::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")] // Serializes as "hash" instead of "Hash"
#[strum(serialize_all = "lowercase")]
pub enum JobType {
    Hash,
    Metadata,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")] // Serializes as "pending" instead of "Pending"
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobEntry {
    pub id: i64,
    pub job_type: JobType,
    pub media_id: Option<i64>,
    pub file_id: Option<i64>,
    pub rel_path: Option<String>,
    pub queued_mtime: Option<i64>,
    pub priority: i32,
    pub status: JobStatus,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl JobEntry {
    pub fn require_file_id(&self) -> AppResult<i64> {
        self.file_id.ok_or_else(|| {
            AppError::Other(format!("Job {} ({:?}) missing file_id", self.id, self.job_type))
        })
    }

    pub fn require_media_id(&self) -> AppResult<i64> {
        self.media_id.ok_or_else(|| {
            AppError::Other(format!("Job {} ({:?}) missing media_id", self.id, self.job_type))
        })
    }
}