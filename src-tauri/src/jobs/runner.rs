use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tracing::{info, debug, error};
use crate::core::constants::{MAX_THREADS, WORKER_THREAD_SLEEP_DURATION};
use crate::core::error::AppResult;
use crate::db;
use crate::jobs::{workers, JobType, JobCompletedPayload, JobStatus};
use crate::jobs::metrics::JobMetrics;

use crate::db::pool::DbManager;


/// Manages the lifecycle of background job worker threads.
///
/// Ensures only one worker runs per library and provides graceful shutdown.
pub struct JobWorkerManager {
    started: AtomicBool,
    shutdown: Arc<AtomicBool>,
    metrics: Arc<JobMetrics>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl Default for JobWorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl JobWorkerManager {
    /// Creates a new JobWorkerManager instance.
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            shutdown: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(JobMetrics::new()),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets the app handle for event emission.
    pub fn set_app_handle(&self, app_handle: AppHandle) {
        if let Ok(mut handle) = self.app_handle.lock() {
            *handle = Some(app_handle);
        }
    }

    /// Returns a reference to the job metrics.
    pub fn metrics(&self) -> &JobMetrics {
        &self.metrics
    }

    /// Attempts to start job worker threads if they haven't been started yet.
    ///
    /// Thread-safe and idempotent. Spawns workers that process Hash, Metadata, and Thumbnail jobs.
    pub fn try_start_worker(&self, library_root: &Path, db_manager: Arc<DbManager>) {
        // Only the first caller that sees `false` gets to start workers.
        if self
            .started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // We won the race, actually spawn worker thread
            info!("Starting job workers for library: {}", library_root.display());
            let app_handle = self.app_handle.clone();
            spawn_job_workers(library_root.to_path_buf(), self.shutdown.clone(), self.metrics.clone(), app_handle, db_manager);
        } else {
            // Already started, no-op
            debug!("Workers already started, skipping");
        }
    }

    /// Signals the worker thread to shut down gracefully after completing its current job.
    pub fn shutdown(&self) {
        info!("Signaling job worker to shut down");
        self.shutdown.store(true, Ordering::SeqCst);
        info!("Job worker shut down gracefully. {}", self.metrics.summary());
    }
}


fn spawn_job_workers(library_root: PathBuf, shutdown: Arc<AtomicBool>, metrics: Arc<JobMetrics>, app_handle: Arc<Mutex<Option<AppHandle>>>, db_manager: Arc<DbManager>) {
    let num_threads = num_cpus::get().min(MAX_THREADS);
    info!("Spawning {} background worker threads", num_threads);

    for i in 0..num_threads {
        let library_root = library_root.clone();
        let shutdown = shutdown.clone();
        let metrics = metrics.clone();
        let app_handle = app_handle.clone();
        let db_manager = db_manager.clone();

        tauri::async_runtime::spawn(async move {
            debug!("Worker thread {} started", i);
            if let Err(e) = worker_loop(library_root, shutdown, metrics, app_handle, db_manager).await {
                error!("Worker thread {} exited with error: {}", i, e);
            }
        });
    }
}

async fn worker_loop(library_root: PathBuf, shutdown: Arc<AtomicBool>, metrics: Arc<JobMetrics>, app_handle: Arc<Mutex<Option<AppHandle>>>, db_manager: Arc<DbManager>) -> AppResult<()> {
    info!("Job worker loop started");

    while !shutdown.load(Ordering::Relaxed) {
        // Claim a job in a short transaction and commit so others see the state change
        let claimed_job = {
            let mut conn = db_manager.get_connection(&library_root)?;
            let tx = conn.transaction()?;
            let job = db::jobs::claim_next_pending(&tx)?;
            tx.commit()?;
            job
        };

        let Some(job) = claimed_job else {
            tokio::time::sleep(WORKER_THREAD_SLEEP_DURATION).await;
            continue;
        };

        info!("Claimed job: id={} type={:?} file_id={:?}", job.id, job.job_type, job.file_id);

        let result = match job.job_type {
            JobType::Metadata => workers::handle_metadata_job(db_manager.clone(), &library_root, &job).await,
            JobType::Thumbnail => workers::handle_thumbnail_job(db_manager.clone(), &library_root, &job).await,
        };

        match result {
            Ok(_) => {
                let mut conn = db_manager.get_connection(&library_root)?;
                let tx = conn.transaction()?;
                db::jobs::mark_job_done(&tx, job.id)?;
                tx.commit()?;

                metrics.record_success(&job.job_type);
                info!("Job completed successfully: id={} type={:?}", job.id, job.job_type);

                if let Ok(hg) = app_handle.lock() {
                    if let Some(ref a) = *hg {
                        let payload = JobCompletedPayload {
                            job_type: job.job_type,
                            media_id: job.media_id,
                            file_id: job.file_id,
                            rel_path: job.rel_path.clone(),
                            status: JobStatus::Done,
                        };
                        let _ = a.emit("job-completed", payload);
                    }
                }
            }
            Err(e) => {
                error!("Job failed: id={} type={:?} error={}", job.id, job.job_type, e);
                metrics.record_failure();

                // ensure the job is marked with an error in a new short transaction
                if let Ok(mut conn) = db_manager.get_connection(&library_root) {
                    if let Ok(tx) = conn.transaction() {
                        let _ = db::jobs::mark_job_error(&tx, job.id, &format!("{e}"));
                        let _ = tx.commit();
                    }
                }

                // Emit event to frontend (even on error, so UI can update)
                if let Ok(hg) = app_handle.lock() {
                    if let Some(ref a) = *hg {
                        let payload = JobCompletedPayload {
                            job_type: job.job_type,
                            media_id: job.media_id,
                            file_id: job.file_id,
                            rel_path: job.rel_path.clone(),
                            status: JobStatus::Error,
                        };
                        let _ = a.emit("job-completed", payload);
                    }
                }
            }
        }
    }
    Ok(())
}
