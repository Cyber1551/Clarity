use std::path::Path;
use std::time::UNIX_EPOCH;
use crate::core::error::AppResult;

pub fn get_file_size(path: &Path) -> AppResult<i64> {
    let meta = std::fs::metadata(path)?;
    Ok(meta.len() as i64)
}

pub fn get_mtime(path: &Path) -> AppResult<i64> {
    let meta = std::fs::metadata(path)?;
    let mtime = meta
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    Ok(mtime)
}