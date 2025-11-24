use std::collections::HashSet;
use std::path::Path;
use rusqlite::{params, OptionalExtension, Transaction};
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

pub fn get_all_files(tx: &Transaction) -> AppResult<Vec<FileEntry>> {
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
            last_scan_mtime,
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
            last_scan_mtime: row.get(8)?,
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

pub fn get_file_by_rel_path(tx: &Transaction, rel_path: &str) -> AppResult<Option<FileEntry>> {
    let mut stmt = tx.prepare(
        r#"
        SELECT
            id,
            media_id,
            dir_path,
            file_name,
            ext,
            size_bytes,
            mtime_secs,
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
                last_scan_mtime: row.get(7)?,
                is_reviewed: row.get::<_, i64>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }).optional()?;

    Ok(existing)
}

pub fn insert_file_row(tx: &Transaction, new_file: &NewFileRecord) -> AppResult<i64> {
    tx.execute(r#"
        INSERT INTO files (
            media_id,
            rel_path,
            dir_path,
            file_name,
            ext,
            size_bytes,
            mtime,
            last_scan_mtime,
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
     "#,
params![
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

pub fn update_file_row(tx: &Transaction, rel_path: &str, size_bytes: &i64, mtime: &i64, now: &i64) -> AppResult<()> {
    tx.execute(r#"
        UPDATE files
        SET
            size_bytes = ?1,
            mtime = ?2,
            last_scan_mtime = ?2,
            updated_at = ?3
        WHERE rel_path = ?4
    "#, params![size_bytes, mtime, now, rel_path])?; // AppError::Database
    Ok(())
}

pub fn update_file_last_seen(tx: &Transaction, mtime: &i64, now: &i64) -> AppResult<()> {
    tx.execute(r#"
        UPDATE files
        SET last_seen_mtime = ?1, updated_at = ?2
    "#, params![mtime, now])?;
    Ok(())
}

pub fn upsert_file(tx: &Transaction, rel_path: &str, full_path: &Path) -> AppResult<UpsertFileResult> {
    let now = now_ms();
    let size_bytes = meta::get_file_size(full_path)?;
    let mtime = meta::get_mtime(full_path)?;
    let existing = get_file_by_rel_path(tx, rel_path)?;

    match existing {
        Some(mut entry) => {
            let mtime_changed = entry.mtime != mtime || entry.size_bytes != size_bytes;

            if mtime_changed {
                update_file_row(tx, &rel_path, &size_bytes, &mtime, &now)?;
                entry.size_bytes = size_bytes;
                entry.mtime = mtime;
            } else {
                // untouched but seen in this scan
                update_file_last_seen(tx, &mtime, &now)?;
            }

            entry.last_scan_mtime = mtime;
            entry.updated_at = now;

            Ok(UpsertFileResult {
                file_entry: entry,
                is_new: false,
                mtime_changed
            })
        }
        None => {
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

            let new_file_id = insert_file_row(tx, &new_file)?;
            let entry = FileEntry {
                id: new_file_id,
                media_id: None,
                rel_path: rel_path.to_string(),
                dir_path: path_components.dir_path,
                file_name: path_components.file_name,
                ext: path_components.ext,
                size_bytes,
                mtime,
                last_scan_mtime: mtime,
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

pub fn delete_file_by_id(trans: &Transaction, file_id: i64) -> AppResult<()> {
    trans.execute(r#"DELETE FROM files WHERE id = ?1"#, params![file_id])?;
    Ok(())
}

pub fn delete_file_by_rel_path(trans: &Transaction, rel_path: &str) -> AppResult<()> {
    trans.execute(r#"DELETE FROM files WHERE rel_path = ?1"#, params![rel_path])?;
    Ok(())
}

pub fn remove_deleted_files(trans: &Transaction, seen_rel_paths: &HashSet<String>) -> AppResult<()> {
    let mut stmt = trans.prepare(r#"
        SELECT id, rel_path
        FROM files
    "#)?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,   // id
            row.get::<_, String>(1)?, // rel_path
        ))
    })?;

    for row in rows {
        let (id, rel_path): (i64, String) = row?;
        if !seen_rel_paths.contains(&rel_path) {
            trans.execute(r#"DELETE FROM files WHERE id = ?1"#, params![id])?;
        }
    }

    Ok(())
}