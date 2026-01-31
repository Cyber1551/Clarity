use crate::core::error::AppResult;
use std::fs;
use std::path::Path;
use crate::core::constants::{OBJECTS_DIRECTORY, LIBRARY_DIRECTORY, IMPORTS_DIRECTORY};

pub fn ensure_core_dirs(root: &Path) -> AppResult<()> {
    let objects = root.join(OBJECTS_DIRECTORY);
    let imports = root.join(IMPORTS_DIRECTORY);
    let library = root.join(LIBRARY_DIRECTORY);

    // Will do nothing if directories already exist
    fs::create_dir_all(&objects)?;
    fs::create_dir_all(&imports)?;
    fs::create_dir_all(&library)?;

    Ok(())
}
