use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    InputOutput(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("file watcher error: {0}")]
    Watcher(#[from] notify::Error),

    #[error("FFmpeg error: {0}")]
    Ffmpeg(String),

    #[error("library root is not configured")]
    LibraryRootMissing,

    #[error("invalid media file: {path:?}")]
    InvalidMedia { path: PathBuf },

    #[error("invalid database state: {0}")]
    InvalidDatabaseState(String),

    #[error("internal invariant violated: {0}")]
    InternalInvariant(String),
}

pub type AppResult<T> = Result<T, AppError>;
