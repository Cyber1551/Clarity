use std::path::Path;
use rusqlite::Transaction;
use crate::core::error::AppResult;
use crate::jobs::JobEntry;

pub fn handle_thumbnail_job(tx: &Transaction, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let media_id_str = job.media_id.map(|id| id.to_string()).unwrap_or("none".to_string());
    println!("(job) handling thumbnail job id={} for media_id={:?}", job.id, media_id_str);

    // TODO: fill in thumbnail ffmpeg pipeline
    Ok(())
}