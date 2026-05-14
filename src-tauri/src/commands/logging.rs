use crate::core::error::AppError;

/// Log severity level forwarded from the frontend logger. Mirrors the `LogLevel` union defined in `src/utils/logger.ts`.
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Frontend-side logger sink.
/// The JS `logger` module fires-and-forgets every log call here so frontend-originated errors land in the same `tracing` stream as backend errors.
/// Returns `()` on success; the frontend never awaits the result so we cannot rely on errors propagating back.
#[tauri::command]
pub fn log_event(
    level: LogLevel,
    scope: String,
    message: String,
    context: Option<String>,
) -> Result<(), AppError> {
    match level {
        LogLevel::Error => tracing::error!(scope = %scope, context = ?context, "{message}"),
        LogLevel::Warn => tracing::warn!(scope = %scope, context = ?context, "{message}"),
        LogLevel::Info => tracing::info!(scope = %scope, context = ?context, "{message}"),
        LogLevel::Debug => tracing::debug!(scope = %scope, context = ?context, "{message}"),
    }
    Ok(())
}
