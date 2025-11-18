use crate::errors::AppResult;
use std::fs;
use std::path::Path;
use crate::core::constants::{OBJECTS_DIRECTORY, SORTED_DIRECTORY, UNSORTED_DIRECTORY};

pub fn ensure_core_dirs(root: &Path) -> AppResult<()> {
    let objects = root.join(OBJECTS_DIRECTORY);
    let unsorted = root.join(UNSORTED_DIRECTORY);
    let sorted = root.join(SORTED_DIRECTORY);

    // Will do nothing if directories already exist
    fs::create_dir_all(&objects)?;
    fs::create_dir_all(&unsorted)?;
    fs::create_dir_all(&sorted)?;

    Ok(())
}
