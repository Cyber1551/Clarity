use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // Serializes as "pending" instead of "Pending"
pub enum JobStatus {
    Pending,
    Processing,
    Done,
    Error
}

// Allows: my_enum.to_string() -> "pending"
impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            JobStatus::Pending => "pending",
            JobStatus::Processing => "processing",
            JobStatus::Done => "done",
            JobStatus::Error => "error",
        };
        write!(f, "{}", s)
    }
}

// Allows: "pending".parse::<JobStatus>()
impl FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(JobStatus::Pending),
            "processing" => Ok(JobStatus::Processing),
            "done" => Ok(JobStatus::Done),
            _ => Ok(JobStatus::Error)
        }
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
    pub job_type: String,
    pub media_id: i64,
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
