use rusqlite::{params, OptionalExtension, Row, Transaction};
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::jobs::{JobEntry, JobStatus, JobType};
use crate::media::MediaEntry;

pub struct EnqueueJobRequest {
    pub file_id: i64,
    pub media_id: Option<i64>,
    pub rel_path: String,
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

pub fn enqueue_job(tx: &Transaction, job_type: JobType, request: &EnqueueJobRequest) -> AppResult<()> {
    let media_id_str = request.media_id.map(|id| id.to_string()).unwrap_or("none".to_string());
    println!("(job) Enqueueing {} job for file id={} with media id {}", job_type, request.file_id, media_id_str);

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
        request.media_id,
        request.file_id,
        request.rel_path,
        request.mtime,
        now_ms()
    ])?;
    Ok(())
}

pub fn claim_next_job(tx: &Transaction) -> AppResult<Option<JobEntry>> {
    let now = now_ms();
    let mut stmt = tx.prepare(r#"
        WITH pick AS (
          SELECT id
          FROM jobs
          WHERE status = 'pending'
          ORDER BY priority DESC, created_at ASC
          LIMIT 1
        )
        UPDATE jobs
        SET
            status = 'processing',
            attempts = attempts + 1,
            updated_at = ?1
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

    let result = stmt.query_row(params![now], map_row_to_job_entry).optional()?;
    Ok(result)
}

pub fn mark_job_done(tx: &Transaction, job_id: i64) -> AppResult<()> {
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