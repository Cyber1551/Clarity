use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc::RecvTimeoutError;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info};
use crate::core::constants::{SORTED_DIRECTORY, WATCHER_DEBOUNCE_DURATION};
use crate::core::error::AppResult;

/// Manages filesystem watching for the Sorted Media directory.
///
/// Detects manual changes and marks the library as dirty.
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
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            dirty: Arc::new(AtomicBool::new(false)),
            shutdown: Arc::new(AtomicBool::new(false)),
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, app_handle: AppHandle) {
        if let Ok(mut handle) = self.app_handle.lock() {
            *handle = Some(app_handle);
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    pub fn try_start_watcher(&self, library_root: &Path) {
        if self
            .started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!("Starting dirty-detection watcher for library: {}", library_root.display());
            spawn_watcher(
                library_root.to_path_buf(),
                self.dirty.clone(),
                self.shutdown.clone(),
                self.app_handle.clone(),
            );
        }
    }

    pub fn shutdown(&self) {
        info!("Shutting down file watcher");
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

fn run_watcher(library_root: &Path, dirty: Arc<AtomicBool>, shutdown: Arc<AtomicBool>, app_handle: Arc<Mutex<Option<AppHandle>>>) -> AppResult<()> {
    let sorted_dir = library_root.join(SORTED_DIRECTORY);

    if !sorted_dir.exists() {
        debug!("Sorted directory does not exist yet: {}", sorted_dir.display());
    }

    let (tx, rx) = std::sync::mpsc::channel::<Result<Event, notify::Error>>();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| crate::core::error::AppError::Other(format!("Failed to create watcher: {e}")))?;

    // Note: We might need to periodically retry watching if SORTED_DIRECTORY doesn't exist yet
    // For now, we assume it's created during init or we'll just fail to watch it until next restart.
    if sorted_dir.exists() {
        watcher.watch(&sorted_dir, RecursiveMode::Recursive)
            .map_err(|e| crate::core::error::AppError::Other(format!("Failed to watch directory: {e}")))?;
    }

    let mut last_event_time: Option<Instant> = None;

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event_result) => {
                match event_result {
                    Ok(_event) => {
                        dirty.store(true, Ordering::Relaxed);
                        last_event_time = Some(Instant::now());
                    }
                    Err(e) => error!("Watcher error: {}", e),
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(last_time) = last_event_time {
                    if last_time.elapsed() >= WATCHER_DEBOUNCE_DURATION && dirty.load(Ordering::Relaxed) {
                        info!("Filesystem change detected in Sorted Media. Marking library as dirty.");
                        if let Ok(handle_guard) = app_handle.lock() {
                            if let Some(ref app) = *handle_guard {
                                let _ = app.emit("library-dirty", true);
                            }
                        }
                        last_event_time = None;
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}
