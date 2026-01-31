use std::sync::atomic::{AtomicU64, Ordering};

/// Tracks metrics for job processing.
#[derive(Debug)]
pub struct JobMetrics {
    jobs_processed: AtomicU64,
    jobs_failed: AtomicU64,
    metadata_jobs_processed: AtomicU64,
    thumbnail_jobs_processed: AtomicU64,
}

impl JobMetrics {
    pub fn new() -> Self {
        Self {
            jobs_processed: AtomicU64::new(0),
            jobs_failed: AtomicU64::new(0),
            metadata_jobs_processed: AtomicU64::new(0),
            thumbnail_jobs_processed: AtomicU64::new(0),
        }
    }

    pub fn record_success(&self, job_type: &crate::jobs::JobType) {
        self.jobs_processed.fetch_add(1, Ordering::Relaxed);

        match job_type {
            crate::jobs::JobType::Metadata => {
                self.metadata_jobs_processed.fetch_add(1, Ordering::Relaxed);
            }
            crate::jobs::JobType::Thumbnail => {
                self.thumbnail_jobs_processed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn record_failure(&self) {
        self.jobs_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn jobs_processed(&self) -> u64 {
        self.jobs_processed.load(Ordering::Relaxed)
    }

    pub fn jobs_failed(&self) -> u64 {
        self.jobs_failed.load(Ordering::Relaxed)
    }

    pub fn metadata_jobs_processed(&self) -> u64 {
        self.metadata_jobs_processed.load(Ordering::Relaxed)
    }

    pub fn thumbnail_jobs_processed(&self) -> u64 {
        self.thumbnail_jobs_processed.load(Ordering::Relaxed)
    }

    /// Returns a summary of all metrics.
    pub fn summary(&self) -> String {
        format!(
            "Jobs: {} processed ({} metadata, {} thumbnail), {} failed",
            self.jobs_processed(),
            self.metadata_jobs_processed(),
            self.thumbnail_jobs_processed(),
            self.jobs_failed()
        )
    }
}

impl Default for JobMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::JobType;

    #[test]
    fn test_job_metrics_initial_state() {
        let metrics = JobMetrics::new();
        assert_eq!(metrics.jobs_processed(), 0);
        assert_eq!(metrics.jobs_failed(), 0);
    }

    #[test]
    fn test_record_success() {
        let metrics = JobMetrics::new();

        metrics.record_success(&JobType::Thumbnail);
        assert_eq!(metrics.jobs_processed(), 1);
        assert_eq!(metrics.thumbnail_jobs_processed(), 1);

        metrics.record_success(&JobType::Metadata);
        assert_eq!(metrics.jobs_processed(), 2);
        assert_eq!(metrics.metadata_jobs_processed(), 1);
    }

    #[test]
    fn test_record_failure() {
        let metrics = JobMetrics::new();

        metrics.record_failure();
        assert_eq!(metrics.jobs_failed(), 1);

        metrics.record_failure();
        assert_eq!(metrics.jobs_failed(), 2);
    }

    #[test]
    fn test_summary() {
        let metrics = JobMetrics::new();
        metrics.record_success(&JobType::Metadata);
        metrics.record_success(&JobType::Thumbnail);
        metrics.record_failure();

        let summary = metrics.summary();
        assert!(summary.contains("2 processed"));
        assert!(summary.contains("1 metadata"));
        assert!(summary.contains("1 thumbnail"));
        assert!(summary.contains("1 failed"));
    }
}
