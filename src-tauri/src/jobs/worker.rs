use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use rusqlite::Transaction;
use crate::core::error::AppResult;
use crate::db;
use crate::db::schema::DbConn;
use crate::jobs::{handle_hash_job, handle_metadata_job, handle_thumbnail_job, JobEntry, JobType};

pub fn spawn_job_worker(library_root: PathBuf) {
    thread::spawn(move || {
        if let Err(e) = worker_loop(library_root) {
            eprintln!("job worker exited with error: {e}");
        }
    });
}

fn worker_loop(library_root: PathBuf) -> AppResult<()> {
    let mut conn = DbConn::new(&library_root)?;
    
    loop {
        println!("(job) worker loop started");
        // Claim a job in a short transaction and commit so others see the state change
        let claimed: Option<JobEntry> = {
            let tx = DbConn::transaction(&mut conn)?;
            let job = db::jobs::claim_next_job(&tx)?;
            tx.commit()?; // ensure 'processing' update is visible immediately
            job
        };

        println!("(job) claimed job: {:?}", claimed);

        match claimed {
            Some(job) => {
                let tx = DbConn::transaction(&mut conn)?;
                // Handle job; ignore individual errors here and mark job as error
                if let Err(e) = handle_job(&tx, &library_root, &job) {
                    eprintln!("job handler error: {e}");
                    db::jobs::mark_job_error(&tx, job.id, &e.to_string())?;
                }
                tx.commit()?;
            }
            None => {
                // No jobs right now, sleep briefly
                thread::sleep(Duration::from_millis(1000));
            }
        }
    }
}


fn handle_job(tx: &Transaction, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let result = match job.job_type {
        JobType::Hash => handle_hash_job(tx, library_root, job),
        JobType::Metadata => handle_metadata_job(tx, library_root, job),
        JobType::Thumbnail => handle_thumbnail_job(tx, library_root, job),
    };

    match result {
        Ok(()) => {
            db::jobs::mark_job_done(tx, job.id)?;
        }
        Err(e) => {
            eprintln!("job {} failed: {}", job.id, e);
            db::jobs::mark_job_error(tx, job.id, &e.to_string())?;
        }
    }

    Ok(())
}