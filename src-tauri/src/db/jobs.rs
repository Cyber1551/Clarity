use rusqlite::{params, Connection, OptionalExtension, Row};
use tracing::{debug, warn};
use crate::core::constants::MAX_JOB_ATTEMPTS;
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::jobs::{JobEntry, JobType};

/// Parameters for enqueueing a background job.
pub struct EnqueueJobRequest {
    pub file_id: i64,
    pub media_id: Option<i64>,
    pub rel_path: String,
    /// Modification time at queue time, used to detect stale jobs
    pub mtime: i64,
}

fn map_row_to_job_entry(row: &Row<'_>) -> rusqlite::Result<JobEntry> {
    Ok(JobEntry {
        id: row.get(0)?,
        job_type: row.get(1)?,
        media_id: row.get(2)?,
        file_id: row.get(3)?,
        rel_path: row.get(4)?,
        queued_mtime: row.get(5)?,
        priority: row.get(6)?,
        status: row.get(7)?,
        attempts: row.get(8)?,
        last_error: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

/// Enqueues a new job into the jobs table.
pub fn enqueue(conn: &Connection, job_type: JobType, request: &EnqueueJobRequest) -> AppResult<()> {
    debug!("Enqueuing {} job for file_id={} media_id={:?}",
           job_type, request.file_id, request.media_id);

    conn.execute(r#"
        INSERT INTO jobs (
            job_type,
            media_id,
            file_id,
            rel_path,
            queued_mtime,
            priority,
            status,
            attempts,
            last_error,
            created_at,
            updated_at
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5,
            0,
            'pending',
            0,
            NULL,
            ?6,
            ?6
        )
    "#,
    params![
        job_type.to_string(),
        request.media_id,
        request.file_id,
        request.rel_path,
        request.mtime,
        now_ms()
    ])?;
    Ok(())
}

/// Claims the next pending job from the queue and marks it as processing.
///
/// Jobs are selected by priority (descending) then creation time (ascending).
/// Automatically retries jobs in 'error' status and cleans up jobs exceeding MAX_JOB_ATTEMPTS.
pub fn claim_next_pending(conn: &Connection) -> AppResult<Option<JobEntry>> {
    // First, clean up jobs that have exceeded max attempts
    cleanup_failed_jobs(conn)?;

    let now = now_ms();
    let mut stmt = conn.prepare(r#"
        WITH pick AS (
          SELECT id
          FROM jobs
          WHERE status IN ('pending', 'error')
            AND attempts < ?1
          ORDER BY priority DESC, created_at DESC
          LIMIT 1
        )
        UPDATE jobs
        SET
            status = 'processing',
            attempts = attempts + 1,
            updated_at = ?2
        WHERE id = (SELECT id FROM pick)
        RETURNING
          id,
          job_type,
          media_id,
          file_id,
          rel_path,
          queued_mtime,
          priority,
          status,
          attempts,
          last_error,
          created_at,
          updated_at
    "#)?;

    let result = stmt.query_row(params![MAX_JOB_ATTEMPTS, now], map_row_to_job_entry).optional()?;
    Ok(result)
}

/// Deletes jobs that have exceeded the maximum number of retry attempts.
fn cleanup_failed_jobs(conn: &Connection) -> AppResult<()> {
    let deleted = conn.execute(r#"
        DELETE FROM jobs
        WHERE attempts >= ?1
          AND status = 'error'
    "#, params![MAX_JOB_ATTEMPTS])?;

    if deleted > 0 {
        warn!("Cleaned up {} jobs that exceeded max retry attempts", deleted);
    }

    Ok(())
}

/// Marks a job as complete by deleting it from the jobs table.
pub fn mark_job_done(conn: &Connection, job_id: i64) -> AppResult<()> {
    conn.execute(r#"
        DELETE FROM jobs
        WHERE id = ?1
    "#, params![job_id])?;
    Ok(())
}

/// Marks a job as failed with an error message.
///
/// The job will be retried if it hasn't exceeded MAX_JOB_ATTEMPTS.
pub fn mark_job_error(conn: &Connection, job_id: i64, msg: &str) -> AppResult<()> {
    let now = now_ms();
    conn.execute(
        r#"
        UPDATE jobs
        SET status = 'error',
            updated_at = ?1,
            last_error = ?2
        WHERE id = ?3
        "#,
        params![now, msg, job_id],
    )?;
    Ok(())
}