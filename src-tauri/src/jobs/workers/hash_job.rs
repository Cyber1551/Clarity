use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;
use crate::core::constants::{HASH_BUFFER_SIZE, UNSORTED_DIRECTORY};
use crate::core::error::{AppResult, ResultExt};
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::db::jobs::EnqueueJobRequest;
use crate::jobs::{helpers, JobEntry, JobType};
use crate::media::MediaType;
use tracing::debug;

use std::sync::Arc;
use crate::db::pool::DbManager;

pub fn handle_hash_job(db_manager: Arc<DbManager>, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let file_id = job.require_file_id()?;
    let file = {
        let mut conn = db_manager.get_connection(library_root)?;
        let tx = conn.transaction()?;
        let f = db::files::get_by_id(&tx, file_id)?;
        tx.commit()?;
        f
    };
    let Some(file) = file else {
        // File gone; just return so job is marked done.
        return Ok(());
    };

    let full_path = library_root.join(&file.rel_path);
    let meta = fs::metadata(&full_path)?;

    let scanned_mtime = file.mtime as u128;
    let current_mtime = meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(scanned_mtime);

    if current_mtime != scanned_mtime {
        // stale job; just return so job is marked done.
        return Ok(());
    }

    // HEAVY WORK: Compute hash outside transaction and WITHOUT holding a connection
    let content_hash = compute_blake3_hash(&full_path)
        .with_context(format!("Failed to compute hash for {}", file.rel_path))?;
    let media_type = MediaType::from_extension(&file.ext);

    // Safe to dedupe outside transaction as it's idempotent and uses content hash
    filesystem::objects::dedupe_to_objects(&full_path, library_root, &content_hash, &file.ext)
        .with_context(format!("Failed to deduplicate file to .objects: {}", file.rel_path))?;

    let now = now_ms();
    
    // START TRANSACTION for final updates - acquire a fresh connection
    let mut conn = db_manager.get_connection(library_root)?;
    let tx = conn.transaction()?;

    let existing_media = db::media::get_by_content_hash(&tx, &content_hash)?;
    let media = if let Some(m) = existing_media.as_ref() {
        m.clone()
    } else {
        db::media::insert_for_hash(&tx, &content_hash, media_type, now)
            .with_context("Failed to create media entry")?
    };

    // If this file is under Unsorted, enforce a single Unsorted link across the entire Unsorted tree for this media.
    if file.dir_path.starts_with(UNSORTED_DIRECTORY) {
        let unsorted_dupes = db::files::list_by_media_in_dir_like(&tx, media.id, UNSORTED_DIRECTORY)?;
        if let Some(other) = unsorted_dupes.into_iter().find(|f| f.id != file.id) {
            debug!(
                current_id = file.id,
                other_id = other.id,
                dir = %file.dir_path,
                "Duplicate Unsorted link for the same media detected; removing current file before assigning media_id"
            );
            let _ = fs::remove_file(&full_path);
            let _ = db::files::delete_by_id(&tx, file.id)?;
            helpers::cleanup_orphaned_media(&tx, library_root, job.media_id)?;
            tx.commit()?;
            return Ok(());
        }
    }

    // Enforce invariant: at most one link per (media_id, dir_path)
    let existing_in_dir = db::files::list_by_media_and_dir(&tx, media.id, &file.dir_path)?;
    if let Some(other) = existing_in_dir.into_iter().find(|f| f.id != file.id) {
        debug!(
            current_id = file.id,
            other_id = other.id,
            dir = %file.dir_path,
            "Duplicate file in the same directory for the same media detected; removing current file before assigning media_id"
        );
        let _ = fs::remove_file(&full_path);
        let _ = db::files::delete_by_id(&tx, file.id)?;
        helpers::cleanup_orphaned_media(&tx, library_root, job.media_id)?;
        tx.commit()?;
        return Ok(());
    }

    // Update media_id and enqueue dependent jobs
    db::files::update_media_id(&tx, file.id, media.id, now)?;

    if existing_media.is_none() || media.metadata_status.is_pending_or_error() || media.thumbnail_status.is_pending_or_error() {
        let req = EnqueueJobRequest { file_id: file.id, media_id: Some(media.id), rel_path: file.rel_path.clone(), mtime: file.mtime };
        db::jobs::enqueue(&tx, JobType::Metadata, &req)?;
        db::jobs::enqueue(&tx, JobType::Thumbnail, &req)?;
    }

    helpers::cleanup_orphaned_media(&tx, library_root, job.media_id)?;
    tx.commit()?;

    Ok(())
}

fn compute_blake3_hash(full_path: &Path) -> AppResult<String> {
    let mut file = fs::File::open(full_path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; HASH_BUFFER_SIZE];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compute_blake3_hash() {
        // Create a temporary file with known content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let hash = compute_blake3_hash(temp_file.path()).unwrap();

        // Verify hash length (blake3 produces 64 hex characters)
        assert_eq!(hash.len(), 64);

        // Verify hash is deterministic
        let hash2 = compute_blake3_hash(temp_file.path()).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_compute_blake3_hash_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let hash = compute_blake3_hash(temp_file.path()).unwrap();

        // Empty file should still produce a valid hash
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_compute_blake3_hash_large_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write 100KB of data (larger than our 64KB buffer)
        let data = vec![0u8; 100_000];
        temp_file.write_all(&data).unwrap();
        temp_file.flush().unwrap();

        let hash = compute_blake3_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }
}
