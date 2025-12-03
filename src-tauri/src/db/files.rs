use std::collections::HashSet;
use std::path::Path;
use rusqlite::{params, OptionalExtension, Transaction};
use tracing::debug;
use crate::filesystem;
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::files::FileEntry;
use crate::filesystem::meta;

#[derive(Debug)]
pub struct NewFileRecord<'a> {
    pub rel_path: &'a str,
    pub dir_path: &'a String,
    pub file_name: &'a String,
    pub ext: &'a String,
    pub size_bytes: i64,
    pub mtime: i64,
    pub now: i64,
}

#[derive(Debug, Clone)]
pub struct UpsertFileResult {
    pub file_entry: FileEntry,
    pub is_new: bool,
    pub mtime_changed: bool
}

pub fn get_all(tx: &Transaction) -> AppResult<Vec<FileEntry>> {
    let mut stmt = tx.prepare(r#"
        SELECT
            id,
            media_id,
            rel_path,
            dir_path,
            file_name,
            ext,
            size_bytes,
            mtime,
            last_seen_mtime,
            is_reviewed,
            created_at,
            updated_at
        FROM files
    "#)?;

    let rows = stmt.query_map([], |row| {
        Ok(FileEntry {
            id: row.get(0)?,
            media_id: row.get(1)?,
            rel_path: row.get(2)?,
            dir_path: row.get(3)?,
            file_name: row.get(4)?,
            ext: row.get(5)?,
            size_bytes: row.get(6)?,
            mtime: row.get(7)?,
            last_seen_mtime: row.get(8)?,
            is_reviewed: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;

    let mut files = Vec::<FileEntry>::new();
    for row in rows {
        files.push(row?);
    }

    Ok(files)
}

pub fn get_by_rel_path(tx: &Transaction, rel_path: &str) -> AppResult<Option<FileEntry>> {
    let mut stmt = tx.prepare(
        r#"
        SELECT
            id,
            media_id,
            dir_path,
            file_name,
            ext,
            size_bytes,
            mtime,
            last_seen_mtime,
            is_reviewed,
            created_at,
            updated_at
        FROM files
        WHERE rel_path = ?1
        "#,
    )?;

    let existing = stmt
        .query_row(params![rel_path], |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                media_id: row.get(1)?,
                rel_path: rel_path.to_string(),
                dir_path: row.get(2)?,
                file_name: row.get(3)?,
                ext: row.get(4)?,
                size_bytes: row.get(5)?,
                mtime: row.get(6)?,
                last_seen_mtime: row.get(7)?,
                is_reviewed: row.get::<_, i64>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).optional()?;

    Ok(existing)
}

/// Retrieves a file entry by its ID.
pub fn get_by_id(tx: &Transaction, file_id: i64) -> AppResult<Option<FileEntry>> {
    let mut stmt = tx.prepare(
        r#"
        SELECT
            id,
            media_id,
            rel_path,
            dir_path,
            file_name,
            ext,
            size_bytes,
            mtime,
            last_seen_mtime,
            is_reviewed,
            created_at,
            updated_at
        FROM files
        WHERE id = ?1
        "#,
    )?;

    let existing = stmt
        .query_row(params![file_id], |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                media_id: row.get(1)?,
                rel_path: row.get(2)?,
                dir_path: row.get(3)?,
                file_name: row.get(4)?,
                ext: row.get(5)?,
                size_bytes: row.get(6)?,
                mtime: row.get(7)?,
                last_seen_mtime: row.get(8)?,
                is_reviewed: row.get::<_, i64>(9)? != 0,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        }).optional()?;

    Ok(existing)
}

pub fn insert(tx: &Transaction, new_file: &NewFileRecord) -> AppResult<i64> {
    tx.execute(r#"
        INSERT INTO files (
            media_id,
            rel_path,
            dir_path,
            file_name,
            ext,
            size_bytes,
            mtime,
            last_seen_mtime,
            is_reviewed,
            created_at,
            updated_at
        ) VALUES (
            NULL,
            ?1, ?2, ?3, ?4,
            ?5, ?6, ?6,
            0,
            ?7, ?7
        )
    "#, params![
            new_file.rel_path,
            new_file.dir_path,
            new_file.file_name,
            new_file.ext,
            new_file.size_bytes,
            new_file.mtime,
            new_file.now,
        ],
    )?;

    let id = tx.last_insert_rowid();
    Ok(id)
}

pub fn update(tx: &Transaction, rel_path: &str, size_bytes: &i64, mtime: &i64, now: &i64) -> AppResult<()> {
    tx.execute(r#"
        UPDATE files
        SET
            size_bytes = ?1,
            mtime = ?2,
            last_seen_mtime = ?2,
            updated_at = ?3
        WHERE rel_path = ?4
    "#, params![size_bytes, mtime, now, rel_path])?; // AppError::Database
    Ok(())
}

pub fn update_last_seen(tx: &Transaction, rel_path: &str, mtime: &i64, now: &i64) -> AppResult<()> {
    tx.execute(r#"
        UPDATE files
        SET last_seen_mtime = ?1,
            updated_at = ?2
        WHERE rel_path = ?3
    "#, params![mtime, now, rel_path])?;
    Ok(())
}

pub fn update_media_id(tx: &Transaction, file_id: i64, media_id: i64, now: i64) -> AppResult<()> {
    tx.execute(r#"
        UPDATE files
        SET media_id = ?1,
            updated_at = ?2
        WHERE id = ?3
    "#, params![media_id, now, file_id])?;
    Ok(())
}

/// Inserts or updates a file record in the database.
///
/// Updates existing files if mtime or size changed, creates new records otherwise.
/// Returns flags indicating if the file is new or modified to help decide if jobs should be enqueued.
pub fn upsert(tx: &Transaction, rel_path: &str, full_path: &Path) -> AppResult<UpsertFileResult> {
    let now = now_ms();
    let size_bytes = meta::get_file_size(full_path)?;
    let mtime = meta::get_mtime(full_path)?;
    debug!("Upserting file: {} (size={}, mtime={})", rel_path, size_bytes, mtime);
    let existing = get_by_rel_path(tx, rel_path)?;

    match existing {
        Some(mut entry) => {
            debug!("File exists in DB: {}", entry.rel_path);
            let mtime_changed = entry.mtime != mtime || entry.size_bytes != size_bytes;

            if mtime_changed {
                debug!("File changed: {} (old_mtime={}, new_mtime={}, old_size={}, new_size={})",
                       rel_path, entry.mtime, mtime, entry.size_bytes, size_bytes);
                update(tx, rel_path, &size_bytes, &mtime, &now)?;
                entry.size_bytes = size_bytes;
                entry.mtime = mtime;
            } else {
                // untouched but seen in this scan
                update_last_seen(tx, rel_path, &mtime, &now)?;
            }

            entry.last_seen_mtime = mtime;
            entry.updated_at = now;

            Ok(UpsertFileResult {
                file_entry: entry,
                is_new: false,
                mtime_changed
            })
        }
        None => {
            debug!("New file discovered: {}", rel_path);
            // Insert a new file row
            let path_components = filesystem::path::split_path(rel_path);

            let new_file = NewFileRecord {
                rel_path,
                dir_path: &path_components.dir_path,
                file_name: &path_components.file_name,
                ext: &path_components.ext,
                size_bytes,
                mtime,
                now,
            };

            let new_file_id = insert(tx, &new_file)?;
            debug!("Inserted new file with id={}", new_file_id);
            let entry = FileEntry {
                id: new_file_id,
                media_id: None,
                rel_path: rel_path.to_string(),
                dir_path: path_components.dir_path,
                file_name: path_components.file_name,
                ext: path_components.ext,
                size_bytes,
                mtime,
                last_seen_mtime: mtime,
                is_reviewed: false,
                created_at: now,
                updated_at: now,
            };

            Ok(UpsertFileResult {
                file_entry: entry,
                is_new: true,
                mtime_changed: true
            })
        }
    }
}

/// Deletes a file entry by its ID.
pub fn delete_by_id(tx: &Transaction, file_id: i64) -> AppResult<()> {
    tx.execute(r#"DELETE FROM files WHERE id = ?1"#, params![file_id])?;
    Ok(())
}

/// Deletes a file entry by its relative path.
pub fn delete_by_rel_path(tx: &Transaction, rel_path: &str) -> AppResult<()> {
    tx.execute(r#"DELETE FROM files WHERE rel_path = ?1"#, params![rel_path])?;
    Ok(())
}

/// Removes files from the database that are not in the seen set.
///
/// Uses an efficient NOT IN query to delete all missing files in one operation.
pub fn remove_deleted_files(tx: &Transaction, seen_rel_paths: &HashSet<String>) -> AppResult<()> {
    if seen_rel_paths.is_empty() {
        // If no files were seen, delete all files
        let deleted = tx.execute("DELETE FROM files", [])?;
        tracing::info!("Removed {} files that no longer exist", deleted);
        return Ok(());
    }

    // For large sets, we need to use a temporary table approach
    // For small-medium sets, we can use IN clause
    if seen_rel_paths.len() < 1000 {
        // Build placeholders for IN clause
        let placeholders = (0..seen_rel_paths.len())
            .map(|i| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(",");

        let query = format!("DELETE FROM files WHERE rel_path NOT IN ({placeholders})");

        // Convert HashSet to Vec for rusqlite
        let rel_paths: Vec<&str> = seen_rel_paths.iter().map(|s| s.as_str()).collect();
        let params = rusqlite::params_from_iter(rel_paths);

        let deleted = tx.execute(&query, params)?;
        if deleted > 0 {
            tracing::info!("Removed {} files that no longer exist", deleted);
        }
    } else {
        // For very large sets, fall back to iterative approach
        let mut stmt = tx.prepare("SELECT id, rel_path FROM files")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut to_delete = Vec::new();
        for row in rows {
            let (id, rel_path): (i64, String) = row?;
            if !seen_rel_paths.contains(&rel_path) {
                to_delete.push(id);
            }
        }

        if !to_delete.is_empty() {
            let placeholders = to_delete.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!("DELETE FROM files WHERE id IN ({placeholders})");
            let params = rusqlite::params_from_iter(to_delete);
            let deleted = tx.execute(&query, params)?;
            tracing::info!("Removed {} files that no longer exist", deleted);
        }
    }

    Ok(())
}