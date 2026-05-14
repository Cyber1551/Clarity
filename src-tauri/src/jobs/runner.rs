use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;
use tracing::{info, debug, warn, error};
use crate::core::constants::{MAX_THREADS, WORKER_THREAD_SLEEP_DURATION};
use crate::core::error::AppResult;
use crate::db;
use crate::jobs::{workers, JobType, JobCompletedPayload, JobStatus, WorkerRecoveredPayload, WorkerStalledPayload};
use crate::jobs::metrics::JobMetrics;

use crate::db::pool::DbManager;

/// Number of consecutive infrastructure failures before the worker is
/// considered stalled and a `worker-stalled` event is emitted to the UI.
const STALL_THRESHOLD: u32 = 3;
/// Linear backoff base. After N consecutive failures the worker waits
/// `min(N * BASE, MAX)` before retrying.
const BACKOFF_BASE_MS: u64 = 1_000;
const BACKOFF_MAX_MS: u64 = 30_000;

fn compute_backoff(consecutive_failures: u32) -> Duration {
    let ms = (consecutive_failures as u64).saturating_mul(BACKOFF_BASE_MS);
    Duration::from_millis(ms.min(BACKOFF_MAX_MS))
}

/// Manages the lifecycle of background job worker threads.
///
/// Ensures only one worker runs per library and provides graceful shutdown.
pub struct JobWorkerManager {
    started: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    metrics: Arc<JobMetrics>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    /// Wakes the worker out of its idle/backoff sleep so a Retry click
    /// doesn't have to wait out the current backoff window.
    wake: Arc<Notify>,
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
            started: Arc::new(AtomicBool::new(false)),
            shutdown: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(JobMetrics::new()),
            app_handle: Arc::new(Mutex::new(None)),
            wake: Arc::new(Notify::new()),
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

    /// Wakes any worker currently in idle/backoff sleep so it tries to
    /// claim/process the next job immediately.
    pub fn wake(&self) {
        self.wake.notify_one();
    }

    /// Attempts to start job worker threads if they haven't been started yet.
    ///
    /// Thread-safe and idempotent. The `started` flag is reset to `false` when
    /// the spawned task exits, so a subsequent call will respawn workers.
    pub fn try_start_worker(&self, library_root: &Path, db_manager: Arc<DbManager>) {
        // Only the first caller that sees `false` gets to start workers.
        if self
            .started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!("Starting job workers for library: {}", library_root.display());
            spawn_job_workers(
                library_root.to_path_buf(),
                self.started.clone(),
                self.shutdown.clone(),
                self.metrics.clone(),
                self.app_handle.clone(),
                self.wake.clone(),
                db_manager,
            );
        } else {
            debug!("Workers already started, skipping");
        }
    }

    /// Signals the worker thread to shut down gracefully after completing its current job.
    pub fn shutdown(&self) {
        info!("Signaling job worker to shut down");
        self.shutdown.store(true, Ordering::SeqCst);
        // Wake any sleeping worker so it observes the shutdown flag immediately.
        self.wake.notify_one();
        info!("Job worker shut down gracefully. {}", self.metrics.summary());
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_job_workers(
    library_root: PathBuf,
    started: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    metrics: Arc<JobMetrics>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    wake: Arc<Notify>,
    db_manager: Arc<DbManager>,
) {
    let num_threads = num_cpus::get().min(MAX_THREADS);
    info!("Spawning {} background worker threads", num_threads);

    for i in 0..num_threads {
        let library_root = library_root.clone();
        let started = started.clone();
        let shutdown = shutdown.clone();
        let metrics = metrics.clone();
        let app_handle = app_handle.clone();
        let wake = wake.clone();
        let db_manager = db_manager.clone();

        tauri::async_runtime::spawn(async move {
            debug!("Worker thread {} started", i);
            let result = worker_loop(library_root, shutdown, metrics, app_handle, wake, db_manager).await;
            if let Err(e) = result {
                error!("Worker thread {} exited with error: {}", i, e);
            } else {
                debug!("Worker thread {} exited cleanly", i);
            }
            // Allow `try_start_worker` to respawn on next call (e.g. via the
            // restart_workers command after the user fixes a stuck DB).
            started.store(false, Ordering::SeqCst);
        });
    }
}

/// Sleeps for `duration` or until `wake` is notified, whichever comes first.
async fn wait_with_signal(duration: Duration, wake: &Notify) {
    tokio::select! {
        _ = tokio::time::sleep(duration) => {}
        _ = wake.notified() => {}
    }
}

/// Emits the `worker-stalled` event to the frontend. Best-effort.
fn emit_stalled(
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
    error_message: String,
    consecutive_failures: u32,
) {
    if let Ok(hg) = app_handle.lock() {
        if let Some(ref a) = *hg {
            let payload = WorkerStalledPayload {
                error_message,
                consecutive_failures,
            };
            let _ = a.emit("worker-stalled", payload);
        }
    }
}

/// Emits the `worker-recovered` event to the frontend. Best-effort.
fn emit_recovered(app_handle: &Arc<Mutex<Option<AppHandle>>>) {
    if let Ok(hg) = app_handle.lock() {
        if let Some(ref a) = *hg {
            let _ = a.emit("worker-recovered", WorkerRecoveredPayload {});
        }
    }
}

/// Runs a fallible infrastructure step. On `Err`, logs the error, increments
/// the consecutive-failure counter, emits `worker-stalled` once the threshold
/// is crossed, sleeps with backoff (interruptible by `wake`), and returns
/// `None` to signal "skip this iteration". On `Ok`, returns the value and (if
/// previously stalled) emits `worker-recovered` and resets state.
async fn run_infra_step<T, F: FnOnce() -> AppResult<T>>(
    label: &str,
    step: F,
    consecutive_failures: &mut u32,
    was_stalled: &mut bool,
    wake: &Notify,
    app_handle: &Arc<Mutex<Option<AppHandle>>>,
) -> Option<T> {
    match step() {
        Ok(value) => {
            if *was_stalled {
                info!("Worker recovered after {} failures", *consecutive_failures);
                emit_recovered(app_handle);
                *was_stalled = false;
            }
            *consecutive_failures = 0;
            Some(value)
        }
        Err(e) => {
            *consecutive_failures = consecutive_failures.saturating_add(1);
            warn!(
                "Worker infra step '{}' failed (attempt {}): {}",
                label, *consecutive_failures, e
            );

            if *consecutive_failures >= STALL_THRESHOLD && !*was_stalled {
                error!(
                    "Worker stalled after {} consecutive failures, last error: {}",
                    *consecutive_failures, e
                );
                emit_stalled(app_handle, format!("{e}"), *consecutive_failures);
                *was_stalled = true;
            }

            let backoff = compute_backoff(*consecutive_failures);
            wait_with_signal(backoff, wake).await;
            None
        }
    }
}

async fn worker_loop(
    library_root: PathBuf,
    shutdown: Arc<AtomicBool>,
    metrics: Arc<JobMetrics>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    wake: Arc<Notify>,
    db_manager: Arc<DbManager>,
) -> AppResult<()> {
    info!("Job worker loop started");

    let mut consecutive_failures: u32 = 0;
    let mut was_stalled: bool = false;

    while !shutdown.load(Ordering::Relaxed) {
        // === Claim phase: fully resilient ===
        let claim_result: Option<Option<_>> = run_infra_step(
            "claim_next_pending",
            || {
                let mut conn = db_manager.get_connection(&library_root)?;
                let tx = conn.transaction()?;
                let job = db::jobs::claim_next_pending(&tx)?;
                tx.commit()?;
                Ok(job)
            },
            &mut consecutive_failures,
            &mut was_stalled,
            &wake,
            &app_handle,
        )
        .await;

        let Some(claimed_job) = claim_result else {
            // Infra step already slept with backoff; loop and try again.
            continue;
        };

        let Some(job) = claimed_job else {
            // No work available: idle wait, interruptible by Retry/wake.
            wait_with_signal(WORKER_THREAD_SLEEP_DURATION, &wake).await;
            continue;
        };

        info!("Claimed job: id={} type={:?} file_id={:?}", job.id, job.job_type, job.file_id);

        let result = match job.job_type {
            JobType::Metadata => workers::handle_metadata_job(db_manager.clone(), &library_root, &job).await,
            JobType::Thumbnail => workers::handle_thumbnail_job(db_manager.clone(), &library_root, &job).await,
        };

        match result {
            Ok(_) => {
                // === Mark-done phase: also resilient ===
                let mark_result = run_infra_step(
                    "mark_job_done",
                    || {
                        let mut conn = db_manager.get_connection(&library_root)?;
                        let tx = conn.transaction()?;
                        db::jobs::mark_job_done(&tx, job.id)?;
                        tx.commit()?;
                        Ok(())
                    },
                    &mut consecutive_failures,
                    &mut was_stalled,
                    &wake,
                    &app_handle,
                )
                .await;

                if mark_result.is_none() {
                    // Couldn't mark done; leave the row in 'processing' and let
                    // the next claim cycle retry. Don't emit job-completed yet.
                    continue;
                }

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

                // Per-job failure: best-effort error annotation. NOT counted against the infrastructure stall threshold
                // That's only for failures of the job-queue plumbing itself.
                if let Ok(mut conn) = db_manager.get_connection(&library_root) {
                    if let Ok(tx) = conn.transaction() {
                        let _ = db::jobs::mark_job_error(&tx, job.id, &format!("{e}"));
                        let _ = tx.commit();
                    }
                }

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
