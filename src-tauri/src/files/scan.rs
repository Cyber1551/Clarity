use std::collections::HashSet;
use std::path::Path;
use tracing::{info, debug, warn};
use walkdir::WalkDir;
use crate::core::constants::UNSORTED_DIRECTORY;
use crate::core::error::AppResult;
use crate::db;
use crate::db::jobs::EnqueueJobRequest;
use crate::db::schema::DbConn;
use crate::filesystem;
use crate::filesystem::path::{get_rel_path, path_to_str};
use crate::jobs::JobType;
use crate::media::{image_utils, video_utils};

/// Statistics tracking reconciliation operations.
#[derive(Debug, Default)]
pub struct ReconcileStats {
    pub new_files: usize,
    pub modified_files: usize,
    pub deleted_files: usize,
    pub unchanged_files: usize,
}

impl ReconcileStats {
    fn summary(&self) -> String {
        format!(
            "new={}, modified={}, deleted={}, unchanged={}",
            self.new_files, self.modified_files, self.deleted_files, self.unchanged_files
        )
    }
}

/// Reconciles the Unsorted Media directory with the database.
///
/// This operation is idempotent - running it twice in a row should do nothing the second time.
///
/// Compares filesystem to the files table and:
/// - **New files**: Inserts file row + enqueues hash job
/// - **Modified files**: Updates mtime/size + enqueues hash job (detected via mtime/size change)
/// - **Deleted files**: Removes file row + runs garbage collection on orphaned media
/// - **Unchanged files**: Updates last_seen_mtime only
///
/// Returns statistics about the reconciliation operation.
pub fn reconcile_unsorted(library_root: &Path) -> AppResult<ReconcileStats> {
    let mut conn = DbConn::new(library_root)?;
    let unsorted_dir = library_root.join(UNSORTED_DIRECTORY);

    if !unsorted_dir.exists() {
        warn!("Unsorted directory does not exist: {}", unsorted_dir.display());
        return Ok(ReconcileStats::default());
    }

    info!("Starting reconciliation of Unsorted Media directory: {}", unsorted_dir.display());
    let mut stats = ReconcileStats::default();
    let mut seen_rel_paths = HashSet::<String>::new();
    let tx = DbConn::transaction(&mut conn)?;

    // Phase 1: Walk filesystem and reconcile with database
    for entry in WalkDir::new(&unsorted_dir).into_iter().filter_map(Result::ok) {
        let file_type = entry.file_type();
        if !file_type.is_file() {
            continue;
        }

        let path = entry.into_path();

        let is_image = image_utils::is_image_file(&path);
        let is_video = video_utils::is_video_file(&path);

        if !(is_image || is_video) {
            continue; // not a supported media file
        }

        let rel_path = get_rel_path(&path, library_root)?;
        let rel_path_str = match path_to_str(&rel_path) {
            Ok(s) => s,
            Err(_) => {
                warn!("Skipping file with invalid UTF-8 path: {:?}", rel_path);
                continue;
            }
        };

        debug!("Reconciling file: {}", rel_path_str);

        let result = db::files::upsert(&tx, &rel_path_str, &path)?;
        seen_rel_paths.insert(rel_path_str.clone());

        if result.is_new {
            info!("New file detected: {}", rel_path_str);
            stats.new_files += 1;

            let file_entry = result.file_entry;
            db::jobs::enqueue(&tx, JobType::Hash, &EnqueueJobRequest {
                file_id: file_entry.id,
                media_id: None,
                rel_path: rel_path_str,
                mtime: file_entry.mtime,
            })?;
        } else if result.mtime_changed {
            info!("Modified file detected: {} (mtime/size changed)", rel_path_str);
            stats.modified_files += 1;

            let file_entry = result.file_entry;
            db::jobs::enqueue(&tx, JobType::Hash, &EnqueueJobRequest {
                file_id: file_entry.id,
                media_id: None,
                rel_path: rel_path_str,
                mtime: file_entry.mtime,
            })?;
        } else {
            // File mtime/size unchanged, but verify .objects integrity for hashed files
            let file_entry = &result.file_entry;

            if let Some(media_id) = file_entry.media_id {
                // File is already hashed, verify canonical .objects file exists
                let media_entry = db::media::get_by_id(&tx, media_id)?;

                if let Some(media) = media_entry {
                    let objects_exists = filesystem::objects::canonical_file_exists(library_root, &media.content_hash);

                    if !objects_exists {
                        warn!("Canonical .objects file missing for {}, re-enqueueing hash job", rel_path_str);
                        stats.modified_files += 1;

                        db::jobs::enqueue(&tx, JobType::Hash, &EnqueueJobRequest {
                            file_id: file_entry.id,
                            media_id: Some(media_id),
                            rel_path: rel_path_str,
                            mtime: file_entry.mtime,
                        })?;
                    } else {
                        debug!("Unchanged file: {}", rel_path_str);
                        stats.unchanged_files += 1;
                    }
                } else {
                    // Media entry missing but file has media_id? Inconsistent state, re-hash
                    warn!("Media entry missing for file {} (media_id={}), re-enqueueing hash job", rel_path_str, media_id);
                    stats.modified_files += 1;

                    db::jobs::enqueue(&tx, JobType::Hash, &EnqueueJobRequest {
                        file_id: file_entry.id,
                        media_id: None,
                        rel_path: rel_path_str,
                        mtime: file_entry.mtime,
                    })?;
                }
            } else {
                // File not yet hashed, mark as unchanged
                debug!("Unchanged file (not yet hashed): {}", rel_path_str);
                stats.unchanged_files += 1;
            }
        }
    }

    // Phase 2: Remove files that no longer exist on disk
    stats.deleted_files = crate::files::gc::remove_deleted_files(&tx, library_root, &seen_rel_paths)?;

    tx.commit()?;
    info!("Reconciliation completed: {}", stats.summary());
    Ok(stats)
}
