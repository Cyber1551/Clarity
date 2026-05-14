use std::path::PathBuf;
use image;
use serde::{Serialize, Serializer, ser::SerializeStruct};
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

    #[error("Migration error: {0}")]
    Refinery(#[from] refinery::Error),

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

    #[error("Media item not found: {0}")]
    NotFound(String),

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
    /// Stable string code for this error variant.
    /// Mirrored on the frontend in `src/api/errors.ts` as the `TauriErrorCode` union,
    /// so callers can branch on the kind of failure rather than scraping the message.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Scan(_) => "SCAN",
            AppError::LibraryRootMissing => "LIBRARY_ROOT_MISSING",
            AppError::Database(_) => "DATABASE",
            AppError::Refinery(_) => "REFINERY",
            AppError::ImageError(_) => "IMAGE",
            AppError::Config(_) => "CONFIG",
            AppError::InputOutput(_) => "IO",
            AppError::Json(_) => "JSON",
            AppError::InvalidFileName { .. } => "INVALID_FILE_NAME",
            AppError::Time => "TIME",
            AppError::FileNotFound(_) => "FILE_NOT_FOUND",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Job(_) => "JOB",
            AppError::HashError(_) => "HASH",
            AppError::DeduplicationError(_) => "DEDUP",
            AppError::MetadataError(_) => "METADATA",
            AppError::ThumbnailError(_) => "THUMBNAIL",
            AppError::Unexpected(_) => "UNEXPECTED",
            AppError::Other(_) => "OTHER",
        }
    }

    /// Optional structured context emitted alongside `code` and `message`.
    /// Only variants carrying extra structured data (beyond the Display string) surface anything here.
    fn context_payload(&self) -> Option<String> {
        match self {
            AppError::InvalidFileName { path } => Some(path.to_string_lossy().into_owned()),
            _ => None,
        }
    }

    /// Logs this error via `tracing::error!`.
    /// Designed to be passed by name to `Result::inspect_err`, so the call-site idiom is `.inspect_err(AppError::log)?`.
    /// Replaces the older `report()` now that commands propagate `AppError` directly to the frontend.
    pub fn log(&self) {
        tracing::error!(code = %self.code(), message = %self, "AppError");
    }

    /// Wraps an error with additional context.
    pub fn context<S: Into<String>>(self, context: S) -> Self {
        let ctx = context.into();
        match self {
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

/// Tauri auto-serializes `Result<T, E>` errors when `E: Serialize`.
/// The shape produced here is the contract consumed by the frontend's `parseTauriError` in `src/api/errors.ts`.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let context = self.context_payload();
        let n_fields = if context.is_some() { 3 } else { 2 };
        let mut s = serializer.serialize_struct("AppError", n_fields)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        if let Some(ctx) = context {
            s.serialize_field("context", &ctx)?;
        }
        s.end()
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
