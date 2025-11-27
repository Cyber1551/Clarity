use std::path::Path;
use rusqlite::Transaction;
use crate::core::error::AppResult;
use crate::jobs::JobEntry;

pub fn handle_hash_job(tx: &Transaction, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let file_id_str = job.file_id.map(|id| id.to_string()).unwrap_or("none".to_string());
    println!("(job) handling hash job id={} for file_id={:?}", job.id, file_id_str);

    // TODO: fill in hash pipeline
    Ok(())
}