use std::path::{Path, PathBuf};
use rusqlite::{params, Connection, Result};
use crate::database::models::MediaItem;
use crate::utils;

/// Get all media items from the database
pub fn get_all_media_items(conn: &Connection) -> Result<Vec<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, file_name, file_size, file_extension, media_type, video_length, hash, created_at, updated_at
         FROM media_items"
    )?;

    let media_items_iter = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;

        Ok(MediaItem {
            id,
            path: PathBuf::from(path),
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_extension: row.get(4)?,
            media_type: row.get(5)?,
            video_length: row.get(6)?,
            hash: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;

    let mut media_items = Vec::new();
    for item in media_items_iter {
        media_items.push(item?);
    }

    Ok(media_items)
}

/// Get a media item by its ID
pub fn get_media_item_by_id(conn: &Connection, id: i64) -> Result<Option<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, file_name, file_size, file_extension, media_type, video_length, hash, created_at, updated_at
         FROM media_items
         WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        let path_string: String = row.get(1)?;

        Ok(Some(MediaItem {
            id: row.get(0)?,
            path: PathBuf::from(path_string),
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_extension: row.get(4)?,
            media_type: row.get(5)?,
            video_length: row.get(6)?,
            hash: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get a media item by its path
pub fn get_media_item_by_path(conn: &Connection, path: &str) -> Result<Option<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, file_name, file_size, file_extension, media_type, video_length, hash, created_at, updated_at
         FROM media_items
         WHERE path = ?1"
    )?;

    let mut rows = stmt.query(params![path])?;

    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let path_string: String = row.get(1)?;

        Ok(Some(MediaItem {
            id,
            path: PathBuf::from(path_string),
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_extension: row.get(4)?,
            media_type: row.get(5)?,
            video_length: row.get(6)?,
            hash: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get all media items with a specific hash (handles duplicates)
pub fn get_media_items_by_hash(conn: &Connection, hash: &str) -> Result<Vec<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, file_name, file_size, file_extension, media_type, video_length, hash, created_at, updated_at
         FROM media_items
         WHERE hash = ?1"
    )?;

    let media_items_iter = stmt.query_map(params![hash], |row| {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;

        Ok(MediaItem {
            id,
            path: PathBuf::from(path),
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_extension: row.get(4)?,
            media_type: row.get(5)?,
            video_length: row.get(6)?,
            hash: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;

    let mut media_items = Vec::new();
    for item in media_items_iter {
        media_items.push(item?);
    }

    Ok(media_items)
}

/// Insert a new media item into the database
pub fn insert_media_item(conn: &Connection, item: &MediaItem) -> Result<i64> {
    let now = utils::get_current_timestamp();

    conn.execute(
        "INSERT INTO media_items (path, file_name, file_size, file_extension, media_type, video_length, hash, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            item.path.to_string_lossy(),
            item.file_name,
            item.file_size,
            item.file_extension,
            item.media_type,
            item.video_length,
            item.hash,
            item.created_at,
            item.updated_at
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Delete a media item and all its associated data by path
pub fn delete_media_item_by_path(conn: &Connection, path: &str) -> Result<()> {
    // Finally, delete the media item itself
    conn.execute(
        "DELETE FROM media_items WHERE path = ?1",
        params![path],
    )?;

    Ok(())
}

/// Update the path of a media item (for handling renamed cache)
pub fn update_media_item_path(conn: &mut Connection, old_path: &str, new_path: &str) -> Result<bool> {
    // First, get the media item ID
    let media_item = match get_media_item_by_path(conn, old_path)? {
        Some(item) => item,
        None => {
            return Ok(false);
        }
    };

    let media_id = &media_item.id;

    // Get the filename from the new path to update the title
    let new_title = Path::new(new_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Update the media item
    let now = utils::get_current_timestamp();
    conn.execute(
        "UPDATE media_items SET path = ?1, file_name = ?2, updated_at = ?3 WHERE id = ?4",
        params![new_path, new_title, now, media_id],
    )?;

    Ok(true)
}

pub fn update_media_item_metadata(conn: &Connection, path: &str, hash: &String, file_size: u64, updated_at: f64) -> Result<bool> {
    // First, get the media item ID
    let media_item = match get_media_item_by_path(conn, path)? {
        Some(item) => item,
        None => {
            return Ok(false);
        }
    };

    let media_id = &media_item.id;

    conn.execute(
        "UPDATE media_items SET hash = ?1, file_size = ?2, updated_at = ?3 WHERE id = ?4",
        params![hash, file_size, updated_at, media_id],
    )?;

    Ok(true)
}
