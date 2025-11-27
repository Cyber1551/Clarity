use rusqlite::{params, OptionalExtension, Transaction};
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::files::FileEntry;
use crate::jobs::{JobEntry, JobStatus, JobType};

pub fn enqueue_job(tx: &Transaction, job_type: JobType, file: &FileEntry) -> AppResult<()> {
    println!("(job) Enqueueing {} job for file id={}", job_type, file.id);
    tx.execute(r#"
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
        file.media_id,
        file.id,
        file.rel_path,
        file.mtime,
        now_ms()
    ])?;
    Ok(())
}

pub fn claim_next_job(tx: &Transaction) -> AppResult<Option<JobEntry>> {
    let now = now_ms();

    // Pick one pending job by priority then oldest first.
    let mut stmt = tx.prepare(
        r#"
        SELECT
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
        FROM jobs
        WHERE status = 'pending'
        ORDER BY priority DESC, created_at ASC
        LIMIT 1
        "#,
    )?;

    let maybe_job = stmt
        .query_row([], |row| {
            let job_type_str: String = row.get(1)?;
            let status_str: String = row.get(7)?;

            Ok(JobEntry {
                id: row.get(0)?,
                job_type: job_type_str.parse::<JobType>().unwrap(),
                media_id: row.get(2)?,
                file_id: row.get(3)?,
                rel_path: row.get(4)?,
                queued_mtime: row.get(5)?,
                priority: row.get(6)?,
                status: status_str.parse::<JobStatus>().unwrap(),
                attempts: row.get(8)?,
                last_error: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        }).optional()?;

    let Some(mut job) = maybe_job else {
        // No jobs, commit and return None
        return Ok(None);
    };

    // Mark as processing + bump attempts
    tx.execute(
        r#"
        UPDATE jobs
        SET status = 'processing',
            attempts = attempts + 1,
            updated_at = ?1
        WHERE id = ?2
        "#,
        params![now, job.id],
    )?;

    job.status = JobStatus::Processing;
    job.attempts += 1;
    job.updated_at = now;

    Ok(Some(job))
}

pub fn mark_job_done(tx: &Transaction, job_id: i64) -> AppResult<()> {
    let now = now_ms();
    tx.execute(r#"
        DELETE FROM jobs
        WHERE id = ?1
    "#, params![job_id])?;
    Ok(())
}

pub fn mark_job_error(tx: &Transaction, job_id: i64, msg: &str) -> AppResult<()> {
    let now = now_ms();
    tx.execute(
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