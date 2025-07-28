use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{MediaItem, Thumbnail};
use crate::state::get_connection;


/// Initialize the database with the required tables
pub fn initialize_tables(conn: &Connection) -> Result<&Connection> {
    let _journal_mode: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get(0))?;
    conn.execute("PRAGMA synchronous = NORMAL;", [])?;

    // Create tables if they don't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS media_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE,
            file_name TEXT,
            file_size INTEGER,
            file_extension TEXT,
            media_type TEXT,
            video_length INTEGER,
            created_at INTEGER,
            updated_at INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS thumbnails (
            media_id INTEGER PRIMARY KEY,
            data BLOB NOT NULL,
            mime_type TEXT,
            FOREIGN KEY (media_id) REFERENCES media_items(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(conn)
}

// Media Item CRUD operations

/// Get all media items from the database
pub fn get_all_media_items(conn: &Connection) -> Result<Vec<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, file_name, file_size, file_extension, media_type, video_length, created_at, updated_at
         FROM media_items"
    )?;

    let media_items_iter = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;

        Ok(MediaItem {
            id,
            path,
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_extension: row.get(4)?,
            media_type: row.get(5)?,
            video_length: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
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
    let now = get_current_timestamp();

    conn.execute(
        "INSERT INTO media_items (path, file_name, file_size, file_extension, media_type, video_length, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            item.path,
            item.file_name,
            item.file_size,
            item.file_extension,
            item.media_type,
            item.video_length,
            now,
            now
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get a media item by its path
pub fn get_media_item_by_path(conn: &Connection, path: &str) -> Result<Option<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, file_name, file_size, file_extension, media_type, video_length, created_at, updated_at
         FROM media_items
         WHERE path = ?1"
    )?;

    let mut rows = stmt.query(params![path])?;

    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;

        Ok(Some(MediaItem {
            id,
            path: row.get(1)?,
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_extension: row.get(4)?,
            media_type: row.get(5)?,
            video_length: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get a media item by its ID
pub fn get_media_item_by_id(conn: &Connection, id: i64) -> Result<Option<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, file_name, file_size, file_extension, media_type, video_length, created_at, updated_at
         FROM media_items
         WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(MediaItem {
            id: row.get(0)?,
            path: row.get(1)?,
            file_name: row.get(2)?,
            file_size: row.get(3)?,
            file_extension: row.get(4)?,
            media_type: row.get(5)?,
            video_length: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }))
    } else {
        Ok(None)
    }
}

// Thumbnail CRUD operations

/// Insert a new thumbnail into the database
pub fn insert_thumbnail(conn: &Connection, thumbnail: &Thumbnail) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO thumbnails (media_id, data, mime_type)
         VALUES (?1, ?2, ?3)",
        params![
            thumbnail.media_id,
            thumbnail.data,
            thumbnail.mime_type
        ],
    )?;

    Ok(())
}

/// Get a thumbnail by its media ID
pub fn get_thumbnail_by_media_id(conn: &Connection, media_id: &i64) -> Result<Option<Thumbnail>> {
    let mut stmt = conn.prepare(
        "SELECT media_id, data, mime_type
         FROM thumbnails
         WHERE media_id = ?1"
    )?;

    let mut rows = stmt.query(params![media_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Thumbnail {
            media_id: row.get(0)?,
            data: row.get(1)?,
            mime_type: row.get(2)?,
        }))
    } else {
        Ok(None)
    }
}


/// Delete a media item and all its associated data by path
pub fn delete_media_item_by_path(conn: &mut Connection, path: &str) -> Result<bool> {
    // First, get the media item ID
    let media_item = match get_media_item_by_path(conn, path)? {
        Some(item) => item,
        None => {
            return Ok(false);
        }
    };

    let media_id = media_item.id.unwrap();

    // Begin a transaction
    let tx = conn.transaction()?;

    // Finally, delete the media item itself
    tx.execute(
        "DELETE FROM media_items WHERE id = ?1",
        params![media_id],
    )?;

    // Commit the transaction
    tx.commit()?;

    Ok(true)
}

/// Update the path of a media item (for handling renamed files)
pub fn update_media_item_path(conn: &mut Connection, old_path: &str, new_path: &str) -> Result<bool> {
    // First, get the media item ID
    let media_item = match get_media_item_by_path(conn, old_path)? {
        Some(item) => item,
        None => {
            return Ok(false);
        }
    };

    let media_id = media_item.id.unwrap();

    // Get the filename from the new path to update the title
    let new_title = Path::new(new_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Update the media item
    let now = get_current_timestamp();
    conn.execute(
        "UPDATE media_items SET path = ?1, title = ?2, updated_at = ?3 WHERE id = ?4",
        params![new_path, new_title, now, media_id],
    )?;

    Ok(true)
}
