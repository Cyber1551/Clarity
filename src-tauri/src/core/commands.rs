use tauri_plugin_dialog::DialogExt;
use crate::core::config::{self, AppConfigDto};
use crate::errors::AppError;
use crate::media::directories;

#[tauri::command]
pub fn get_app_config(app: tauri::AppHandle) -> Result<AppConfigDto, String> {
    let app_config = config::load_config(&app).map_err(|e: AppError| e.to_string())?;
    Ok(AppConfigDto::from(app_config))
}

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

#[tauri::command]
pub fn initialize_library_dirs(app: tauri::AppHandle) -> Result<(), String> {
    let root = config::get_library_root(&app).map_err(|e: AppError| e.to_string())?;
    directories::ensure_core_dirs(&root).map_err(|e: AppError| e.to_string())?;
    Ok(())
}