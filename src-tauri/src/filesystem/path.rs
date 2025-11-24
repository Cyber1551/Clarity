use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use crate::core::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct PathComponents {
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
}

pub fn path_to_str(path: &Path) -> AppResult<String> {
    // Normalize the path (for example, `.` and `..`)
    let cleaned: PathBuf = path.components().collect();

    // Convert to UTF-8 string
    let s = match cleaned.to_str() {
        Some(s) => s,
        None => return Err(AppError::InvalidFileName {
            path: path.to_path_buf(),
        }),
    };

    let normalized = s.replace('\\', "/");
    Ok(normalized)
}

pub fn get_extension(path: &Path) -> &str {
    path.extension().and_then(OsStr::to_str).unwrap_or("unknown")
}

pub fn get_rel_path(path: &Path, library_root: &Path) -> AppResult<PathBuf> {
    match path.strip_prefix(&library_root) {
        Ok(rel) => Ok(rel.to_path_buf()),
        Err(_) => Err(AppError::LibraryRootMissing)
    }
}

pub fn split_path(path: &str) -> PathComponents {
    let path = Path::new(path);

    let file_name_os = path.file_name().unwrap_or_default();
    let file_name_str = file_name_os.to_string_lossy();

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let file_stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name_str.to_string());

    let dir_path = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string());

    PathComponents {
        dir_path,
        file_name: file_stem,
        ext,
    }
}