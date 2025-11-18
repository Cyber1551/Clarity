use crate::core::constants::CONFIG_FILE_NAME;
use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub library_root: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigDto {
    pub library_root: Option<String>,
}

impl From<AppConfig> for AppConfigDto {
    fn from(cfg: AppConfig) -> Self {
        Self {
            library_root: cfg.library_root,
        }
    }
}

fn config_path(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .ok()
        .ok_or_else(|| {
            AppError::InternalInvariant("No application config directory available".to_string())
        })?;

    fs::create_dir_all(&dir)?; // AppError::Io
    Ok(dir.join(CONFIG_FILE_NAME))
}

pub fn load_config(app: &AppHandle) -> AppResult<AppConfig> {
    let path = config_path(app)?;
    println!("Loading configuration from {}", path.display());
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let bytes = fs::read(&path)?; // AppError::Io
    let cfg = serde_json::from_slice(&bytes)?; // AppError::Json
    Ok(cfg)
}

pub fn save_config(app: &AppHandle, cfg: &AppConfig) -> AppResult<()> {
    let path = config_path(app)?;
    let json = serde_json::to_vec_pretty(cfg)?; // AppError::Json
    fs::write(&path, json)?; // AppError::Io
    Ok(())
}

pub fn get_library_root(app: &AppHandle) -> AppResult<PathBuf> {
    let cfg: AppConfig = load_config(app)?;
    let Some(root_str) = cfg.library_root else {
        return Err(AppError::LibraryRootMissing);
    };

    Ok(PathBuf::from(root_str))
}
