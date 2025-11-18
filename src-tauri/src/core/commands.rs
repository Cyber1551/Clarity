use crate::core::config::{self, AppConfigDto};
use crate::errors::AppError;
use crate::media::directory_utils;
use tauri_plugin_dialog::DialogExt;

/// - Ok(AppConfigDta) returns the configuration data for the app (such as library root folder)
/// - Err(String) on error
#[tauri::command]
pub fn get_app_config(app: tauri::AppHandle) -> Result<AppConfigDto, String> {
    let app_config = config::load_config(&app).map_err(|e: AppError| e.to_string())?;
    Ok(AppConfigDto::from(app_config))
}

/// - Ok(Some(path)) if the user picked a folder, and it was saved
/// - Ok(None) if the user canceled
/// - Err(String) on error
#[tauri::command]
pub async fn choose_library_root(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder();

    let Some(folder_path) = folder else {
        return Ok(None);
    };

    let folder_str = folder_path.to_string();

    let mut cfg = config::load_config(&app).map_err(|e: AppError| e.to_string())?;
    cfg.library_root = Some(folder_str.clone());
    config::save_config(&app, &cfg).map_err(|e: AppError| e.to_string())?;

    Ok(Some(folder_str))
}

/// Initializes the library directories required for the application's functionality.
/// - Ok(()) on success. Idempotent. Will succeed regardless if the folders were missing or already created
/// - Err(String) on error
#[tauri::command]
pub fn initialize_library(app: tauri::AppHandle) -> Result<(), String> {
    let root = config::get_library_root(&app).map_err(|e: AppError| e.to_string())?;
    directory_utils::ensure_core_dirs(&root).map_err(|e: AppError| e.to_string())?;
    Ok(())
}
