use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use crate::core::constants::WORKER_THREAD_SLEEP_DURATION;
use crate::core::error::AppResult;
use crate::db;
use crate::db::schema::DbConn;
use crate::jobs::{workers, JobType};

pub struct JobWorkerManager {
    started: AtomicBool,
}

impl JobWorkerManager {
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
        }
    }

    /// Start a job worker if it hasn't been started yet.
    pub fn try_start_worker(&self, library_root: &Path) {
        // Only the first caller that sees `false` gets to start workers.
        if self
            .started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // We won the race, actually spawn worker thread
            println!("Starting job worker");
            spawn_job_worker(library_root.to_path_buf());
        } else {
            // Already started, no-op
            println!("Worker already started, skipping");
        }
    }
}


fn spawn_job_worker(library_root: PathBuf) {
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
        let tx = DbConn::transaction(&mut conn)?;
        let claimed_job = db::jobs::claim_next_job(&tx)?;
        let Some(job) = claimed_job else {
            tx.commit()?;
            thread::sleep(WORKER_THREAD_SLEEP_DURATION);
            continue;
        };

        println!("(job) claimed job: {:?}", job);

        let result = match job.job_type {
            JobType::Hash => workers::handle_hash_job(&tx, &library_root, &job),
            JobType::Metadata => workers::handle_metadata_job(&tx, &library_root, &job),
            JobType::Thumbnail => workers::handle_thumbnail_job(&tx, &library_root, &job),
        };

        match result {
            Ok(_) => {
                db::jobs::mark_job_done(&tx, job.id)?;
                tx.commit()?;
            }
            Err(e) => {
                tx.rollback()?;
                // ensure the job is marked with an error in a new short transaction
                let tx1 = conn.transaction()?;
                db::jobs::mark_job_error(&tx1, job.id, &format!("{e}"))?;
                tx1.commit()?;
            }
        }
    }
}