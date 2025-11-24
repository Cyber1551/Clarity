use std::collections::HashSet;
use std::path::Path;
use rusqlite::Transaction;
use walkdir::WalkDir;
use crate::core::constants::UNSORTED_DIRECTORY;
use crate::core::error::AppResult;
use crate::db;
use crate::db::schema::DbConn;
use crate::filesystem::path::{get_rel_path, path_to_str};
use crate::media::{image_utils, video_utils};

pub fn scan_unsorted(library_root: &Path) -> AppResult<()> {
    let mut conn = DbConn::new(library_root)?;
    let unsorted_dir = library_root.join(UNSORTED_DIRECTORY);
    if !unsorted_dir.exists() {
        // Missing unsorted directory, nothing to scan
        return Ok(());
    }

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

        let rel_path = get_rel_path(&path, &library_root)?;
        let rel_path_str = match path_to_str(&rel_path) {
            Ok(s) => s,
            Err(_) => continue, // This will happen if the path contains invalid UTF-8 characters. Skip the file
        };

        let result = db::files::upsert_file(&tx, &rel_path_str, &path)?;

        seen_rel_paths.insert(rel_path_str);

        if result.is_new || result.mtime_changed {
            // new or changed file detected: queue a job
            db::jobs::enqueue_hash_job(&tx, &result.file_entry)?;
        }
    }

    delete_missing_files(&tx, &seen_rel_paths)?;

    tx.commit()?;
    Ok(())
}

fn delete_missing_files(tx: &Transaction, seen_rel_paths: &HashSet<String>) -> AppResult<()> {
    let all_files = db::files::get_all_files(tx)?;

    for file in all_files {
        let rel_path = file.rel_path;
        if !seen_rel_paths.contains(&rel_path) {
            db::files::delete_file_by_rel_path(tx, &rel_path)?;
        }
    }

    Ok(())
}
