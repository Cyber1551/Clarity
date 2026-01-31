use rusqlite::{params, Connection, OptionalExtension};
use tracing::{debug, warn};
use crate::core::constants::MAX_JOB_ATTEMPTS;
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::jobs::{JobRow, JobType, EnqueueJobRequest};

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
pub fn claim_next_pending(conn: &Connection) -> AppResult<Option<JobRow>> {
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

    let result = stmt.query_row(params![MAX_JOB_ATTEMPTS, now], JobRow::from_row).optional()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::initialize_schema;

    #[test]
    fn test_enqueue_and_claim() -> AppResult<()> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn).map_err(|e| crate::core::error::AppError::Database(e))?;

        // Disable foreign keys for this test to avoid needing to populate media/media_links tables
        conn.execute("PRAGMA foreign_keys = OFF", [])?;

        let req = EnqueueJobRequest {
            file_id: 1,
            media_id: Some(10),
            rel_path: "test/path.jpg".to_string(),
            mtime: 12345,
        };

        enqueue(&conn, JobType::Thumbnail, &req)?;

        let claimed = claim_next_pending(&conn)?;
        assert!(claimed.is_some());
        let job = claimed.unwrap();
        assert_eq!(job.job_type, JobType::Thumbnail);
        assert_eq!(job.file_id, Some(1));
        assert_eq!(job.media_id, Some(10));
        assert_eq!(job.status, crate::jobs::JobStatus::Processing);
        assert_eq!(job.attempts, 1);

        Ok(())
    }

    #[test]
    fn test_job_error_and_retry() -> AppResult<()> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn).map_err(|e| crate::core::error::AppError::Database(e))?;
        conn.execute("PRAGMA foreign_keys = OFF", [])?;

        let req = EnqueueJobRequest {
            file_id: 1,
            media_id: Some(10),
            rel_path: "test/path.jpg".to_string(),
            mtime: 12345,
        };
        enqueue(&conn, JobType::Metadata, &req)?;

        let job = claim_next_pending(&conn)?.unwrap();
        mark_job_error(&conn, job.id, "some error")?;

        let retried = claim_next_pending(&conn)?.unwrap();
        assert_eq!(retried.id, job.id);
        assert_eq!(retried.attempts, 2);
        assert_eq!(retried.status, crate::jobs::JobStatus::Processing);

        Ok(())
    }

    #[test]
    fn test_job_max_attempts_cleanup() -> AppResult<()> {
        let conn = Connection::open_in_memory()?;
        initialize_schema(&conn).map_err(|e| crate::core::error::AppError::Database(e))?;
        conn.execute("PRAGMA foreign_keys = OFF", [])?;

        let req = EnqueueJobRequest {
            file_id: 1,
            media_id: Some(10),
            rel_path: "test/path.jpg".to_string(),
            mtime: 12345,
        };
        enqueue(&conn, JobType::Metadata, &req)?;

        // Exhaust attempts
        for _ in 0..MAX_JOB_ATTEMPTS {
            let job = claim_next_pending(&conn)?.unwrap();
            mark_job_error(&conn, job.id, "error")?;
        }

        // Next claim should cleanup the failed job and return None (since there's only one)
        let claimed = claim_next_pending(&conn)?;
        assert!(claimed.is_none());

        // Verify it was deleted
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0))?;
        assert_eq!(count, 0);

        Ok(())
    }
}