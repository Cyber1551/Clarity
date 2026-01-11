use tauri::{Emitter, State};
use tracing::info;
use crate::core::config::{self, AppConfigDto};
use crate::core::error::AppError;
use tauri_plugin_dialog::DialogExt;
use crate::files::scan::reconcile_unsorted;
use crate::filesystem::directory;
use crate::filesystem::watcher::FileWatcherManager;
use crate::jobs::runner::JobWorkerManager;
use crate::db::pool::DbManager;
use crate::core::state::LibraryRootState;

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
pub async fn choose_library_root(app: tauri::AppHandle, job_worker_manager: State<'_, JobWorkerManager>, watcher_manager: State<'_, FileWatcherManager>, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<Option<String>, String> {
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
            watcher_manager.try_start_watcher(path);
        }
        None => {
            return Ok(None)
        }
    };

    Ok(Some(folder_str))
}

/// Initializes the library and reconciles the Unsorted Media directory.
/// - Ok(()) on success. Idempotent. Running multiple times only processes changes.
/// - Err(String) on error
#[tauri::command]
pub async fn initialize_library(app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, _library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<(), String>  {
    let root = config::get_library_root(&app).map_err(|e: AppError| e.report())?;
    directory::ensure_core_dirs(&root).map_err(|e: AppError| e.report())?;

    let mut conn = db_manager.get_connection(&root).map_err(|e: AppError| e.report())?;
    let stats = reconcile_unsorted(&mut conn, &root).map_err(|e: AppError| e.report())?;
    info!("Library initialization complete: new={}, modified={}, deleted={}, unchanged={}",
          stats.new_files, stats.modified_files, stats.deleted_files, stats.unchanged_files);

    let _ = app.emit("library-initialized", ());

    Ok(())
}
