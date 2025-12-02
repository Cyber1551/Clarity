use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;
use rusqlite::Transaction;
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::db::jobs::EnqueueJobRequest;
use crate::jobs::{helpers, JobEntry, JobType};
use crate::media::MediaType;

pub fn handle_hash_job(tx: &Transaction, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let file_id = job.require_file_id()?;
    let Some(file) = db::files::get_file_by_id(tx, &file_id)? else {
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

    let content_hash = compute_blake3_hash(&full_path)?;
    let media_type = MediaType::from_extension(&file.ext);

    let now = now_ms();
    let existing_media = db::media::get_by_content_hash(tx, &content_hash)?;
    let media = if let Some(m) = existing_media.as_ref() {
        m.clone()
    } else {
        db::media::insert_media_for_hash(tx, &content_hash, media_type, now)?
    };

    db::files::update_file_media_id(tx, file.id, media.id, now)?;

    // Canonicalize in .objects/ with hardlink
    filesystem::objects::dedupe_to_objects(&full_path, library_root, &content_hash, &file.ext)?;

    // Enqueue only if needed
    if existing_media.is_none() || media.metadata_status.is_pending_or_error() || media.thumbnail_status.is_pending_or_error() {
        let req = EnqueueJobRequest { file_id, media_id: Some(media.id), rel_path: file.rel_path.clone(), mtime: file.mtime };
        db::jobs::enqueue_job(tx, JobType::Metadata, &req)?;
        db::jobs::enqueue_job(tx, JobType::Thumbnail, &req)?;
    }

    helpers::cleanup_orphaned_media(tx, library_root, job.media_id)?;

    Ok(())
}

fn compute_blake3_hash(full_path: &Path) -> AppResult<String> {
    let mut file = fs::File::open(full_path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}