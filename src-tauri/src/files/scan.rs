use std::collections::HashSet;
use std::path::Path;
use tracing::{info, debug, warn};
use walkdir::WalkDir;
use crate::core::constants::UNSORTED_DIRECTORY;
use crate::core::error::AppResult;
use crate::db;
use crate::db::jobs::EnqueueJobRequest;
use crate::db::schema::DbConn;
use crate::filesystem::path::{get_rel_path, path_to_str};
use crate::jobs::JobType;
use crate::media::{image_utils, video_utils};

/// Scans the unsorted directory for new or modified media files.
///
/// Walks the directory tree, creates/updates database entries, and enqueues Hash jobs
/// for new or changed files. Removes entries for files that no longer exist.
pub fn scan_unsorted(library_root: &Path) -> AppResult<()> {
    let mut conn = DbConn::new(library_root)?;
    let unsorted_dir = library_root.join(UNSORTED_DIRECTORY);

    if !unsorted_dir.exists() {
        warn!("Unsorted directory does not exist: {}", unsorted_dir.display());
        return Ok(());
    }

    info!("Starting scan of unsorted directory: {}", unsorted_dir.display());
    let mut seen_rel_paths = HashSet::<String>::new();
    let tx = DbConn::transaction(&mut conn)?;

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

        debug!("Found media file: {}", rel_path_str);

        let result = db::files::upsert(&tx, &rel_path_str, &path)?;
        seen_rel_paths.insert(rel_path_str.clone());

        if result.is_new || result.mtime_changed {
            info!("New or changed file detected: {} (new={}, mtime_changed={})",
                  rel_path_str, result.is_new, result.mtime_changed);

            let file_entry = result.file_entry;
            db::jobs::enqueue(&tx, JobType::Hash, &EnqueueJobRequest {
                file_id: file_entry.id,
                media_id: None,
                rel_path: rel_path_str,
                mtime: file_entry.mtime,
            })?;
        }
    }

    db::files::remove_deleted_files(&tx, &seen_rel_paths)?;

    tx.commit()?;
    info!("Scan completed. Total files found: {}", seen_rel_paths.len());
    Ok(())
}
