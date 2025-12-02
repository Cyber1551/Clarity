use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tracing::{info, debug, error};
use crate::core::constants::WORKER_THREAD_SLEEP_DURATION;
use crate::core::error::AppResult;
use crate::db;
use crate::db::schema::DbConn;
use crate::jobs::{workers, JobType};
use crate::jobs::metrics::JobMetrics;

/// Manages the lifecycle of background job worker threads.
///
/// Ensures only one worker runs per library and provides graceful shutdown.
pub struct JobWorkerManager {
    started: AtomicBool,
    shutdown: Arc<AtomicBool>,
    metrics: Arc<JobMetrics>,
}

impl JobWorkerManager {
    /// Creates a new JobWorkerManager instance.
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            shutdown: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(JobMetrics::new()),
        }
    }

    /// Returns a reference to the job metrics.
    pub fn metrics(&self) -> &JobMetrics {
        &self.metrics
    }

    /// Attempts to start a job worker thread if one hasn't been started yet.
    ///
    /// Thread-safe and idempotent. Spawns a worker that processes Hash, Metadata, and Thumbnail jobs sequentially.
    pub fn try_start_worker(&self, library_root: &Path) {
        // Only the first caller that sees `false` gets to start workers.
        if self
            .started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // We won the race, actually spawn worker thread
            info!("Starting job worker for library: {}", library_root.display());
            spawn_job_worker(library_root.to_path_buf(), self.shutdown.clone(), self.metrics.clone());
        } else {
            // Already started, no-op
            debug!("Worker already started, skipping");
        }
    }

    /// Signals the worker thread to shut down gracefully after completing its current job.
    pub fn shutdown(&self) {
        info!("Signaling job worker to shut down");
        self.shutdown.store(true, Ordering::SeqCst);
        info!("Job worker shut down gracefully. {}", self.metrics.summary());
    }
}


fn spawn_job_worker(library_root: PathBuf, shutdown: Arc<AtomicBool>, metrics: Arc<JobMetrics>) {
    thread::spawn(move || {
        if let Err(e) = worker_loop(library_root, shutdown, metrics) {
            error!("Job worker exited with error: {}", e);
        }
    });
}

fn worker_loop(library_root: PathBuf, shutdown: Arc<AtomicBool>, metrics: Arc<JobMetrics>) -> AppResult<()> {
    let mut conn = DbConn::new(&library_root)?;

    info!("Job worker loop started");

    while !shutdown.load(Ordering::Relaxed) {
        debug!("Checking for pending jobs");

        // Claim a job in a short transaction and commit so others see the state change
        let tx = DbConn::transaction(&mut conn)?;
        let claimed_job = db::jobs::claim_next_job(&tx)?;
        let Some(job) = claimed_job else {
            tx.commit()?;
            thread::sleep(WORKER_THREAD_SLEEP_DURATION);
            continue;
        };

        info!("Claimed job: id={} type={:?} file_id={:?}", job.id, job.job_type, job.file_id);

        let result = match job.job_type {
            JobType::Hash => workers::handle_hash_job(&tx, &library_root, &job),
            JobType::Metadata => workers::handle_metadata_job(&tx, &library_root, &job),
            JobType::Thumbnail => workers::handle_thumbnail_job(&tx, &library_root, &job),
        };

        match result {
            Ok(_) => {
                db::jobs::mark_job_done(&tx, job.id)?;
                tx.commit()?;
                metrics.record_success(&job.job_type);
                info!("Job completed successfully: id={} type={:?}", job.id, job.job_type);
            }
            Err(e) => {
                error!("Job failed: id={} type={:?} error={}", job.id, job.job_type, e);
                metrics.record_failure();
                tx.rollback()?;
                // ensure the job is marked with an error in a new short transaction
                let tx1 = conn.transaction()?;
                db::jobs::mark_job_error(&tx1, job.id, &format!("{e}"))?;
                tx1.commit()?;
            }
        }
        info!("Current Metrics: {}", metrics.summary());
    }

    Ok(())
}