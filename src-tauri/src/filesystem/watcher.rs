use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc::RecvTimeoutError;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{debug, error, info, warn};
use crate::core::constants::{UNSORTED_DIRECTORY, WATCHER_DEBOUNCE_DURATION};
use crate::core::error::AppResult;
use crate::files::scan::reconcile_unsorted;
use crate::db::pool::DbManager;

/// Manages filesystem watching for the Unsorted Media directory.
///
/// Detects changes and triggers reconciliation after debouncing.
pub struct FileWatcherManager {
    started: AtomicBool,
    dirty: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl Default for FileWatcherManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FileWatcherManager {
    /// Creates a new FileWatcherManager instance.
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            dirty: Arc::new(AtomicBool::new(false)),
            shutdown: Arc::new(AtomicBool::new(false)),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets the app handle for event emission and reconciliation.
    pub fn set_app_handle(&self, app_handle: AppHandle) {
        if let Ok(mut handle) = self.app_handle.lock() {
            *handle = Some(app_handle);
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    /// Attempts to start the file watcher for the given library root.
    ///
    /// Thread-safe and idempotent.
    pub fn try_start_watcher(&self, library_root: &Path) {
        if self
            .started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!("Starting file watcher for library: {}", library_root.display());
            spawn_watcher(
                library_root.to_path_buf(),
                self.dirty.clone(),
                self.shutdown.clone(),
                self.app_handle.clone(),
            );
        } else {
            debug!("File watcher already started, skipping");
        }
    }

    pub fn shutdown(&self) {
        info!("Signaling file watcher to shut down");
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

fn spawn_watcher(library_root: PathBuf, dirty: Arc<AtomicBool>, shutdown: Arc<AtomicBool>, app_handle: Arc<Mutex<Option<AppHandle>>>) {
    thread::spawn(move || {
        if let Err(e) = run_watcher(&library_root, dirty, shutdown, app_handle) {
            error!("File watcher exited with error: {}", e);
        }
    });
}

/// Runs the file watcher loop with debouncing.
fn run_watcher(library_root: &Path, dirty: Arc<AtomicBool>, shutdown: Arc<AtomicBool>, app_handle: Arc<Mutex<Option<AppHandle>>>) -> AppResult<()> {
    let unsorted_dir = library_root.join(UNSORTED_DIRECTORY);

    if !unsorted_dir.exists() {
        warn!("Cannot start watcher: Unsorted directory does not exist: {}", unsorted_dir.display());
        return Ok(());
    }

    info!("Watching directory: {}", unsorted_dir.display());

    let (tx, rx) = std::sync::mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| crate::core::error::AppError::Other(format!("Failed to create watcher: {e}")))?;

    watcher.watch(&unsorted_dir, RecursiveMode::Recursive)
        .map_err(|e| crate::core::error::AppError::Other(format!("Failed to watch directory: {e}")))?;

    let mut last_event_time: Option<Instant> = None;

    loop {
        if shutdown.load(Ordering::Relaxed) {
            info!("File watcher shutting down");
            break;
        }

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event_result) => {
                match event_result {
                    Ok(event) => {
                        debug!("Filesystem event: {:?}", event.kind);
                        dirty.store(true, Ordering::Relaxed);
                        last_event_time = Some(Instant::now());
                    }
                    Err(e) => {
                        error!("Watcher error: {}", e);
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(last_time) = last_event_time {
                    if last_time.elapsed() >= WATCHER_DEBOUNCE_DURATION && dirty.load(Ordering::Relaxed) {
                        info!("Debounce elapsed, triggering reconciliation");

                        if let Ok(handle_guard) = app_handle.lock() {
                            if let Some(ref app) = *handle_guard {
                                let res = (|| {
                                    let db_manager = app.state::<Arc<DbManager>>();
                                    let mut conn = db_manager.get_connection(library_root)?;
                                    let stats = reconcile_unsorted(&mut conn, library_root)?;
                                    Ok::<_, Box<dyn std::error::Error>>(stats)
                                })();

                                match res {
                                    Ok(stats) => {
                                        info!("Reconciliation complete: new={}, modified={}, deleted={}, unchanged={}",
                                              stats.new_files, stats.modified_files, stats.deleted_files, stats.unchanged_files);

                                        if stats.new_files > 0 || stats.modified_files > 0 || stats.deleted_files > 0 {
                                            if let Err(e) = app.emit("library-changed", ()) {
                                                error!("Failed to emit library-changed event: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Reconciliation failed: {}", e);
                                    }
                                }
                            }
                        }

                        dirty.store(false, Ordering::Relaxed);
                        last_event_time = None;
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                warn!("Watcher channel disconnected");
                break;
            }
        }
    }

    Ok(())
}
