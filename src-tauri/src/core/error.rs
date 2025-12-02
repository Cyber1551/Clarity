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

    #[error("Image error {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Config error {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    InputOutput(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Filename is not valid UTF-8: {path:?}")]
    InvalidFileName { path: PathBuf },

    #[error("Invalid parsed time")]
    Time,

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl AppError {
    /// Returns a string representation of the error for logging purposes.
    pub fn report(&self) -> String {
        tracing::error!("AppError occurred: {}", self);
        self.to_string()
    }
}
