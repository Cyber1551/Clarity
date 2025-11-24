use rusqlite::{params, Transaction};
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::files::FileEntry;

pub fn enqueue_hash_job(tx: &Transaction, file: &FileEntry) -> AppResult<()> {
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
            'hash',
            COALESCE(?1, 0),
            ?2,
            ?3,
            ?4,
            0,
            'pending',
            0,
            NULL,
            ?5,
            ?5
        )
    "#,
    params![
        file.media_id,
        file.id,
        file.rel_path,
        file.mtime,
        now_ms()
    ])?;
    Ok(())
}
