use std::collections::HashSet;
use std::path::Path;
use rusqlite::Transaction;
use tracing::{info, debug};
use crate::core::error::AppResult;
use crate::db;
use crate::filesystem;

/// Garbage collects orphaned media after a file deletion.
///
/// Deletes media row, thumbnail, and .objects/ file if no other media_files reference this media.
/// Returns true if collection occurred, false if media is still referenced.
pub fn collect_orphaned_media(tx: &Transaction, library_root: &Path, media_id: i64) -> AppResult<bool> {
    let deleted_hash = db::media::delete_unreferenced_by_id(tx, media_id)?;

    if let Some(content_hash) = deleted_hash {
        info!("Garbage collecting orphaned media: media_id={}, content_hash={}", media_id, content_hash);
        filesystem::objects::remove_canonical_objects_file(library_root, &content_hash)?;
        Ok(true)
    } else {
        debug!("Media media_id={} still referenced, skipping GC", media_id);
        Ok(false)
    }
}

/// Garbage collects all orphaned media in the database.
///
/// Finds and removes all media entries with no file references.
/// Returns the number of entries collected.
pub fn collect_all_orphaned_media(tx: &Transaction, library_root: &Path) -> AppResult<usize> {
    let orphaned_media = db::media::get_orphaned_media(tx)?;

    let count = orphaned_media.len();
    if count == 0 {
        return Ok(0);
    }

    info!("Garbage collecting {} orphaned media entries", count);

    for (media_id, content_hash) in orphaned_media {
        db::media::delete_unreferenced_by_id(tx, media_id)?;
        filesystem::objects::remove_canonical_objects_file(library_root, &content_hash)?;
    }

    Ok(count)
}

/// Deletes a file by ID and performs garbage collection on orphaned media.
pub fn delete_file_by_id(tx: &Transaction, library_root: &Path, file_id: i64) -> AppResult<()> {
    let deleted_file = db::media_files::delete_by_id(tx, file_id)?;

    if let Some(file) = deleted_file {
        let collected = collect_orphaned_media(tx, library_root, file.media_id)?;
        if collected {
            info!("Garbage collected orphaned media after deleting file_id={}", file_id);
        }
    }

    Ok(())
}

/// Deletes a file by relative path and performs garbage collection on orphaned media.
pub fn delete_file_by_rel_path(tx: &Transaction, library_root: &Path, rel_path: &str) -> AppResult<()> {
    let deleted_file = db::media_files::delete_by_rel_path(tx, rel_path)?;

    if let Some(file) = deleted_file {
        let collected = collect_orphaned_media(tx, library_root, file.media_id)?;
        if collected {
            info!("Garbage collected orphaned media after deleting file at path={}", rel_path);
        }
    }

    Ok(())
}

/// Removes deleted media_files from a specific directory subtree (dir_path LIKE `${dir_prefix}%`)
/// and performs garbage collection on orphaned media.
///
/// Only media_files under the provided dir_prefix are considered for deletion. This prevents
/// accidental removal of media_files from other projections (e.g., Library/ tag folders) when
/// reconciling Unsorted.
///
/// Returns the number of media_files deleted under the scoped directory.
pub fn remove_deleted_files_in_dir(
    tx: &Transaction,
    library_root: &Path,
    dir_prefix: &str,
    seen_rel_paths: &HashSet<String>,
) -> AppResult<usize> {
    let deleted_count = db::media_files::remove_deleted_files_in_dir_like(tx, dir_prefix, seen_rel_paths)?;

    if deleted_count > 0 {
        info!("Removed {} media_files that no longer exist", deleted_count);
    }

    let gc_count = collect_all_orphaned_media(tx, library_root)?;
    if gc_count > 0 {
        info!("Garbage collected {} orphaned media entries after bulk file deletion", gc_count);
    }

    Ok(deleted_count)
}
