use tauri::{Emitter, State};
use crate::core::config::{self, AppConfigDto};
use crate::core::error::AppError;
use tauri_plugin_dialog::DialogExt;
use crate::filesystem::directory;
use crate::jobs::runner::JobWorkerManager;
use crate::db::pool::DbManager;
use crate::core::state::LibraryRootState;
use tauri_plugin_opener::OpenerExt;

use std::sync::Arc;

/// - Ok(AppConfigDta) returns the configuration data for the app (such as library root folder)
/// - Err(String) on error
#[tauri::command]
pub fn get_app_config(app: tauri::AppHandle, _job_worker_manager: State<JobWorkerManager>) -> Result<AppConfigDto, String> {
    let app_config = config::load_config(&app).map_err(|e: AppError| e.report())?;
    Ok(AppConfigDto::from(app_config))
}

/// - Ok(Some(path)) if the user picked a folder, and it was saved
/// - Ok(None) if the user canceled
/// - Err(String) on error
#[tauri::command]
pub async fn choose_library_root(app: tauri::AppHandle, job_worker_manager: State<'_, JobWorkerManager>, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<Option<String>, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder();

    let Some(folder_path) = folder else {
        return Ok(None);
    };

    let folder_str = folder_path.to_string();

    match folder_path.as_path() {
        Some(path) => {
            let mut cfg = config::load_config(&app).map_err(|e: AppError| e.report())?;
            cfg.library_root = Some(folder_str.clone());
            config::save_config(&app, &cfg).map_err(|e: AppError| e.report())?;
            *library_root_state.0.lock().unwrap() = Some(path.to_path_buf());

            job_worker_manager.try_start_worker(path, db_manager.inner().clone());
        }
        None => {
            return Ok(None)
        }
    };

    Ok(Some(folder_str))
}

/// Initializes the library directory structure.
/// - Ok(()) on success. Idempotent.
/// - Err(String) on error
#[tauri::command]
pub async fn initialize_library(app: tauri::AppHandle) -> Result<(), String>  {
    let root = config::get_library_root(&app).map_err(|e: AppError| e.report())?;
    directory::ensure_core_dirs(&root).map_err(|e: AppError| e.report())?;

    let _ = app.emit("library-initialized", ());

    Ok(())
}

/// Opens the library root in the system default file explorer.
#[tauri::command]
pub fn open_library_root(app: tauri::AppHandle) -> Result<(), String> {
    let root = config::get_library_root(&app).map_err(|e: AppError| e.report())?;

    app.opener().open_path(root.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())?;

    Ok(())
}

/// User-initiated worker restart. Wakes any worker currently in backoff
/// (so it retries immediately) and respawns the worker if a previous task
/// died. Used by the stalled-worker UI banner's Retry button.
#[tauri::command]
pub fn restart_workers(
    job_worker_manager: State<'_, JobWorkerManager>,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<(), String> {
    let root = library_root_state
        .0
        .lock()
        .unwrap()
        .as_ref()
        .ok_or_else(|| "Library root not set".to_string())?
        .clone();

    job_worker_manager.wake();
    job_worker_manager.try_start_worker(&root, db_manager.inner().clone());
    Ok(())
}
