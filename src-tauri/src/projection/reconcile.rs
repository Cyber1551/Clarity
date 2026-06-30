use std::collections::HashSet;
use std::fs;
use std::path::Path;
use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info};

use crate::core::constants::{IMPORTS_DIRECTORY, LIBRARY_DIRECTORY};
use crate::core::error::{AppError, AppResult};
use crate::core::time::now_ms;
use crate::db;
use crate::filesystem::objects;
use super::paths;

#[derive(Debug, Default, Clone, Serialize)]
pub struct SyncReport {
    pub reconciled: usize,
    pub created: usize,
    pub removed: usize,
}

/// Idempotently materializes a reviewed item's attributes into the `Library/` hardlink tree:
/// Diffs desired links against existing ones, drops the `Imports/` staging links, and sets `projected_at`.
/// Returns (created, removed) counts.
pub fn reconcile_media(conn: &Connection, root: &Path, media_id: i64) -> AppResult<(usize, usize)> {
    let media = db::media::get_by_id(conn, media_id)?
        .ok_or_else(|| AppError::NotFound("media not found".into()))?;

    if media.reviewed_at.is_none() {
        return Ok((0, 0));
    }

    let tags = db::tags::list_for_media(conn, media_id)?;
    let canonical = objects::find_canonical_objects_file(root, &media.content_hash)?;
    let ext = canonical
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    let base = paths::base_name(&media);

    let lib_prefix = format!("{}/", LIBRARY_DIRECTORY);
    let current = db::media_files::list_by_media_in_dir_like(conn, media_id, &lib_prefix)?;

    // Resolve every desired Library link to a concrete rel_path (with collision handling).
    let mut desired: HashSet<String> = HashSet::new();
    for dir in paths::desired_dirs(&media, &tags) {
        let rel = resolve_rel_path(conn, media_id, &dir, &base, &ext, &media.content_hash)?;
        desired.insert(rel);
    }

    let mut created = 0usize;
    let mut removed = 0usize;

    // Remove undesired links first or it trips the (media_id, dir_path) unique index since a rename keeps the same dir_path.
    for f in &current {
        if desired.contains(&f.rel_path) {
            continue;
        }
        let abs = root.join(&f.rel_path);
        let _ = fs::remove_file(&abs);
        db::media_files::delete_by_id(conn, f.id)?;
        removed += 1;
    }

    for rel in &desired {
        if current.iter().any(|f| &f.rel_path == rel) {
            continue;
        }
        let abs = root.join(rel);
        create_hardlink(&canonical, &abs)?;
        db::media_files::upsert(conn, media_id, rel, &abs)?;
        created += 1;
    }

    // A reviewed item no longer belongs in the Imports staging inbox.
    let import_prefix = format!("{}/", IMPORTS_DIRECTORY);
    let import_links = db::media_files::list_by_media_in_dir_like(conn, media_id, &import_prefix)?;
    for f in &import_links {
        let abs = root.join(&f.rel_path);
        let _ = fs::remove_file(&abs);
        db::media_files::delete_by_id(conn, f.id)?;
        removed += 1;
    }

    db::media::set_projected_at(conn, media_id, now_ms())?;
    debug!("reconciled media_id={media_id}: +{created} -{removed}");
    Ok((created, removed))
}

/// Reconciles every reviewed item that is dirty (changed since its last projection).
pub fn sync_library(conn: &Connection, root: &Path) -> AppResult<SyncReport> {
    let ids = db::media::get_dirty_reviewed(conn)?;
    let report = reconcile_ids(conn, root, &ids)?;
    cleanup_empty_dirs(&root.join(LIBRARY_DIRECTORY));
    info!(
        "library sync: reconciled {} items (+{} -{})",
        report.reconciled, report.created, report.removed
    );
    Ok(report)
}

/// Wipes the entire `Library/` tree (disk + rows) and reprojects every reviewed item from scratch.
pub fn rebuild_library(conn: &Connection, root: &Path) -> AppResult<SyncReport> {
    let lib_abs = root.join(LIBRARY_DIRECTORY);
    if lib_abs.exists() {
        fs::remove_dir_all(&lib_abs).map_err(AppError::from)?;
    }
    db::media_files::delete_by_dir_like(conn, &format!("{}/", LIBRARY_DIRECTORY))?;

    let ids = db::media::get_reviewed_ids(conn)?;
    let report = reconcile_ids(conn, root, &ids)?;
    info!(
        "library rebuild: reconciled {} items (+{} -{})",
        report.reconciled, report.created, report.removed
    );
    Ok(report)
}

fn reconcile_ids(conn: &Connection, root: &Path, ids: &[i64]) -> AppResult<SyncReport> {
    let mut report = SyncReport::default();
    for &id in ids {
        let (created, removed) = reconcile_media(conn, root, id)?;
        report.reconciled += 1;
        report.created += created;
        report.removed += removed;
    }
    Ok(report)
}

/// Picks the concrete rel_path for one (media, dir) pair, appending a content-hash suffix when the base name is already taken by a *different* media in that folder.
fn resolve_rel_path(
    conn: &Connection,
    media_id: i64,
    dir: &str,
    base: &str,
    ext: &str,
    content_hash: &str,
) -> AppResult<String> {
    let candidate = format!("{dir}/{}", paths::join_name(base, ext));
    match db::media_files::get_by_rel_path(conn, &candidate)? {
        Some(row) if row.media_id != media_id => {
            Ok(format!("{dir}/{}", paths::collision_name(base, ext, content_hash)))
        }
        _ => Ok(candidate),
    }
}

fn create_hardlink(canonical: &Path, target: &Path) -> AppResult<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(AppError::from)?;
    }
    if target.exists() {
        fs::remove_file(target).map_err(AppError::from)?;
    }
    fs::hard_link(canonical, target).map_err(AppError::from)?;
    Ok(())
}

/// Recursively removes empty directories under `dir` (and `dir` itself if it ends up empty).
/// Best-effort: filesystem errors are ignored so cleanup never fails a sync.
fn cleanup_empty_dirs(dir: &Path) {
    let _ = remove_if_empty(dir);
}

fn remove_if_empty(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };

    let mut is_empty = true;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if !remove_if_empty(&path) {
                is_empty = false;
            }
        } else {
            is_empty = false;
        }
    }

    if is_empty {
        fs::remove_dir(dir).is_ok()
    } else {
        false
    }
}
