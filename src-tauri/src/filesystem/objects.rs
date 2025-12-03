use std::fs;
use std::path::{Path, PathBuf};
use crate::core::constants::OBJECTS_DIRECTORY;
use crate::core::error::{AppError, AppResult};

/// Deduplicates a file to the .objects directory using hard links.
///
/// If this is the first time seeing this content hash, moves the file to .objects.
/// Otherwise, removes the file and creates a hard link to the canonical .objects file.
pub fn dedupe_to_objects(full_path: &Path, library_root: &Path, content_hash: &str, ext: &str, ) -> AppResult<()> {
    let canonical_path = canonical_objects_path(library_root, content_hash, ext);

    // Ensure .objects exists (should already)
    if let Some(parent) = canonical_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !canonical_path.exists() {
        // First time we've seen this content: move file to .objects
        fs::rename(full_path, &canonical_path)?;
    } else {
        // Canonical already exists: remove this full_path before recreating as hardlink
        if full_path.exists() {
            fs::remove_file(full_path)?;
        }
    }

    // Now create a hard link from canonical back to Unsorted path
    fs::hard_link(&canonical_path, full_path)?;

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