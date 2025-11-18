use std::ffi::OsStr;
use std::path::{Path};
use crate::errors::AppError;

pub fn path_to_utf8_string(path: &Path) -> Result<String, AppError> {
    match path.to_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(AppError::InvalidFileName {
            path: path.to_path_buf()
        })
    }
}

pub fn get_extension(path: &Path) -> &str {
    path.extension().and_then(OsStr::to_str).unwrap_or("unknown")
}
