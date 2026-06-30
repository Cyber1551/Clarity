use std::sync::Arc;
use tauri::{Emitter, State};
use crate::core::error::AppError;
use crate::core::state::LibraryRootState;
use crate::db;
use crate::db::pool::DbManager;
use crate::projection::{self, SyncReport};
use super::dto::SyncStatusDto;

/// Materializes all dirty reviewed items into the Library hardlink tree.
#[tauri::command]
pub fn sync_library(
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<SyncReport, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let report = projection::sync_library(&conn, root).inspect_err(AppError::log)?;

    let _ = app.emit("library-changed", ());
    Ok(report)
}

/// Wipes the Library tree and reprojects every reviewed item from scratch.
#[tauri::command]
pub fn rebuild_library(
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<SyncReport, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let report = projection::rebuild_library(&conn, root).inspect_err(AppError::log)?;

    let _ = app.emit("library-changed", ());
    Ok(report)
}

/// Returns how many reviewed items are dirty (pending projection).
#[tauri::command]
pub fn get_sync_status(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<SyncStatusDto, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let dirty_count = db::media::count_dirty(&conn).inspect_err(AppError::log)?;
    Ok(SyncStatusDto { dirty_count })
}
