use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::jobs::worker::spawn_job_worker;

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
