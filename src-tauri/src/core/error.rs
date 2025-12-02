use std::path::PathBuf;
use image;
use thiserror::Error;

/// Result type alias using AppError.
pub type AppResult<T> = Result<T, AppError>;

/// Application-wide error type.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Scan error: {0}")]
    Scan(String),

    #[error("Library root missing")]
    LibraryRootMissing,

    #[error("SQLite error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Config error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    InputOutput(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Filename is not valid UTF-8: {path:?}")]
    InvalidFileName { path: PathBuf },

    #[error("Invalid parsed time")]
    Time,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Job error: {0}")]
    Job(String),

    #[error("Hash computation failed: {0}")]
    HashError(String),

    #[error("Deduplication error: {0}")]
    DeduplicationError(String),

    #[error("Metadata extraction failed: {0}")]
    MetadataError(String),

    #[error("Thumbnail generation failed: {0}")]
    ThumbnailError(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl AppError {
    /// Returns a string representation of the error for logging purposes.
    pub fn report(&self) -> String {
        tracing::error!("AppError occurred: {}", self);
        self.to_string()
    }

    /// Wraps an error with additional context.
    pub fn context<S: Into<String>>(self, context: S) -> Self {
        let ctx = context.into();
        match self {
            // Preserve specific error types when adding context
            AppError::Job(msg) => AppError::Job(format!("{ctx}: {msg}")),
            AppError::HashError(msg) => AppError::HashError(format!("{ctx}: {msg}")),
            AppError::DeduplicationError(msg) => AppError::DeduplicationError(format!("{ctx}: {msg}")),
            AppError::MetadataError(msg) => AppError::MetadataError(format!("{ctx}: {msg}")),
            AppError::ThumbnailError(msg) => AppError::ThumbnailError(format!("{ctx}: {msg}")),
            AppError::Other(msg) => AppError::Other(format!("{ctx}: {msg}")),
            _ => AppError::Other(format!("{ctx}: {self}")),
        }
    }
}

/// Extension trait for adding context to Results.
pub trait ResultExt<T> {
    /// Adds context to an error if the Result is Err.
    fn with_context<S: Into<String>>(self, context: S) -> AppResult<T>;
}

impl<T> ResultExt<T> for AppResult<T> {
    fn with_context<S: Into<String>>(self, context: S) -> AppResult<T> {
        self.map_err(|e| e.context(context))
    }
}
