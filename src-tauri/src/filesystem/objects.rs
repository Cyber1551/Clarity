use std::fs;
use std::path::{Path, PathBuf};
use crate::core::constants::OBJECTS_DIRECTORY;
use crate::core::error::{AppError, AppResult};

/// Ingests a file from an external source into the .objects store and
/// creates a hardlink projection at the target path.
pub fn ingest_and_link(
    source_path: &Path,
    library_root: &Path,
    content_hash: &str,
    ext: &str,
    projection_path: &Path
) -> AppResult<()> {
    let canonical_path = canonical_objects_path(library_root, content_hash, ext);

    // 1. Ensure canonical version exists in .objects
    if !canonical_path.exists() {
        if let Some(parent) = canonical_path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Copy directly from source to Source of Truth (.objects)
        fs::copy(source_path, &canonical_path)?;
    }

    // 2. Create the projection (hardlink) in the library
    if projection_path.exists() {
        fs::remove_file(projection_path)?;
    }

    // Ensure parent of projection path exists
    if let Some(parent) = projection_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::hard_link(&canonical_path, projection_path)?;

    Ok(())
}

/// Removes the canonical file from the .objects directory for a given content hash.
pub fn remove_canonical_objects_file(library_root: &Path, content_hash: &str) -> AppResult<()> {
    if let Some(path) = find_object_by_hash(library_root, content_hash) {
        // Ignore errors; if this fails, it's just a leftover file.
        let _ = fs::remove_file(&path);
    }

    Ok(())
}

/// Checks if the canonical file exists in .objects for a given content hash.
pub fn canonical_file_exists(library_root: &Path, content_hash: &str) -> bool {
    find_object_by_hash(library_root, content_hash).is_some()
}

/// Finds the canonical file path in .objects for a given content hash.
pub fn find_canonical_objects_file(library_root: &Path, content_hash: &str) -> AppResult<PathBuf> {
    find_object_by_hash(library_root, content_hash)
        .ok_or_else(|| AppError::FileNotFound(format!("canonical object for hash {content_hash} not found in {OBJECTS_DIRECTORY:?} folder")))
}

fn canonical_objects_path(library_root: &Path, content_hash: &str, ext: &str) -> PathBuf {
    let objects_dir = library_root.join(OBJECTS_DIRECTORY);
    if ext.is_empty() {
        objects_dir.join(content_hash)
    } else {
        objects_dir.join(format!("{content_hash}.{ext}"))
    }
}

fn find_object_by_hash(library_root: &Path, content_hash: &str) -> Option<PathBuf> {
    let objects_dir = library_root.join(OBJECTS_DIRECTORY);

    fs::read_dir(&objects_dir).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.starts_with(content_hash) {
            Some(entry.path())
        } else {
            None
        }
    })
}