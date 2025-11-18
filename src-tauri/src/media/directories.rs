use crate::errors::AppResult;
use std::fs;
use std::path::Path;

pub fn ensure_core_dirs(root: &Path) -> AppResult<()> {
    let objects = root.join(".objects");
    let unsorted = root.join("Unsorted");
    let sorted = root.join("Sorted");

    // Will do nothing if directories already exist
    fs::create_dir_all(&objects)?;
    fs::create_dir_all(&unsorted)?;
    fs::create_dir_all(&sorted)?;

    Ok(())
}
