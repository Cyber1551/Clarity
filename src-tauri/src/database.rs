use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{MediaItem, Thumbnail, Tag, Bookmark};

/// Initialize the database with the required tables
pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Create tables if they don't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS media_items (
            id INTEGER PRIMARY KEY,
            path TEXT UNIQUE,
            title TEXT,
            type TEXT,
            length INTEGER,
            created_at INTEGER,
            updated_at INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS thumbnails (
            id INTEGER PRIMARY KEY,
            media_id INTEGER,
            data BLOB,
            mime_type TEXT,
            FOREIGN KEY (media_id) REFERENCES media_items(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS media_tags (
            media_id INTEGER,
            tag_id INTEGER,
            PRIMARY KEY (media_id, tag_id),
            FOREIGN KEY (media_id) REFERENCES media_items(id),
            FOREIGN KEY (tag_id) REFERENCES tags(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY,
            media_id INTEGER,
            description TEXT,
            timestamp INTEGER,
            FOREIGN KEY (media_id) REFERENCES media_items(id)
        )",
        [],
    )?;

    Ok(conn)
}

/// Get the current timestamp in seconds since the Unix epoch
pub fn get_current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// Media Item CRUD operations

/// Insert a new media item into the database
pub fn insert_media_item(conn: &Connection, item: &MediaItem) -> Result<i64> {
    let now = get_current_timestamp();

    conn.execute(
        "INSERT INTO media_items (path, title, type, length, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            item.path,
            item.title,
            item.media_type,
            item.length,
            now,
            now
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get a media item by its path
pub fn get_media_item_by_path(conn: &Connection, path: &str) -> Result<Option<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, type, length, created_at, updated_at 
         FROM media_items 
         WHERE path = ?1"
    )?;

    let mut rows = stmt.query(params![path])?;

    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;

        Ok(Some(MediaItem {
            id: Some(id),
            path: row.get(1)?,
            title: row.get(2)?,
            media_type: row.get(3)?,
            length: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get all media items from the database
pub fn get_all_media_items(conn: &Connection) -> Result<Vec<MediaItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, type, length, created_at, updated_at 
         FROM media_items"
    )?;

    let media_items_iter = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let path: String = row.get(1)?;

        Ok(MediaItem {
            id: Some(id),
            path,
            title: row.get(2)?,
            media_type: row.get(3)?,
            length: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut media_items = Vec::new();
    for item in media_items_iter {
        media_items.push(item?);
    }

    Ok(media_items)
}

// Thumbnail CRUD operations

/// Insert a new thumbnail into the database
pub fn insert_thumbnail(conn: &Connection, thumbnail: &Thumbnail) -> Result<i64> {
    conn.execute(
        "INSERT INTO thumbnails (media_id, data, mime_type)
         VALUES (?1, ?2, ?3)",
        params![
            thumbnail.media_id,
            thumbnail.data,
            thumbnail.mime_type
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get a thumbnail by its media ID
pub fn get_thumbnail_by_media_id(conn: &Connection, media_id: i64) -> Result<Option<Thumbnail>> {
    let mut stmt = conn.prepare(
        "SELECT id, media_id, data, mime_type 
         FROM thumbnails 
         WHERE media_id = ?1"
    )?;

    let mut rows = stmt.query(params![media_id])?;

    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;

        Ok(Some(Thumbnail {
            id: Some(id),
            media_id: row.get(1)?,
            data: row.get(2)?,
            mime_type: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get a thumbnail by its ID
pub fn get_thumbnail_by_id(conn: &Connection, thumbnail_id: i64) -> Result<Option<Thumbnail>> {
    let mut stmt = conn.prepare(
        "SELECT id, media_id, data, mime_type 
         FROM thumbnails 
         WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![thumbnail_id])?;

    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;

        Ok(Some(Thumbnail {
            id: Some(id),
            media_id: row.get(1)?,
            data: row.get(2)?,
            mime_type: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

// Tag CRUD operations

/// Insert a new tag into the database or get an existing one
pub fn insert_tag(conn: &Connection, tag: &Tag) -> Result<i64> {
    // Try to find existing tag first
    if let Some(existing_tag) = get_tag_by_name(conn, &tag.name)? {
        return Ok(existing_tag.id.unwrap());
    }

    conn.execute(
        "INSERT INTO tags (name) VALUES (?1)",
        params![tag.name],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get a tag by its name
pub fn get_tag_by_name(conn: &Connection, name: &str) -> Result<Option<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name FROM tags WHERE name = ?1"
    )?;

    let mut rows = stmt.query(params![name])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Tag {
            id: Some(row.get(0)?),
            name: row.get(1)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get all tags for a media item
pub fn get_tags_for_media(conn: &Connection, media_id: i64) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name 
         FROM tags t
         JOIN media_tags mt ON t.id = mt.tag_id
         WHERE mt.media_id = ?1"
    )?;

    let tags_iter = stmt.query_map(params![media_id], |row| {
        Ok(Tag {
            id: Some(row.get(0)?),
            name: row.get(1)?,
        })
    })?;

    let mut tags = Vec::new();
    for tag in tags_iter {
        tags.push(tag?);
    }

    Ok(tags)
}

/// Add a tag to a media item
pub fn add_tag_to_media(conn: &Connection, media_id: i64, tag_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO media_tags (media_id, tag_id) VALUES (?1, ?2)",
        params![media_id, tag_id],
    )?;

    Ok(())
}

// Bookmark CRUD operations

/// Insert a new bookmark into the database
pub fn insert_bookmark(conn: &Connection, bookmark: &Bookmark) -> Result<i64> {
    conn.execute(
        "INSERT INTO bookmarks (media_id, description, timestamp)
         VALUES (?1, ?2, ?3)",
        params![
            bookmark.media_id,
            bookmark.description,
            bookmark.timestamp
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get all bookmarks for a media item
pub fn get_bookmarks_for_media(conn: &Connection, media_id: i64) -> Result<Vec<Bookmark>> {
    let mut stmt = conn.prepare(
        "SELECT id, media_id, description, timestamp 
         FROM bookmarks 
         WHERE media_id = ?1"
    )?;

    let bookmarks_iter = stmt.query_map(params![media_id], |row| {
        Ok(Bookmark {
            id: Some(row.get(0)?),
            media_id: row.get(1)?,
            description: row.get(2)?,
            timestamp: row.get(3)?,
        })
    })?;

    let mut bookmarks = Vec::new();
    for bookmark in bookmarks_iter {
        bookmarks.push(bookmark?);
    }

    Ok(bookmarks)
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

    // Delete associated bookmarks
    tx.execute(
        "DELETE FROM bookmarks WHERE media_id = ?1",
        params![media_id],
    )?;

    // Delete associated tags (from the junction table)
    tx.execute(
        "DELETE FROM media_tags WHERE media_id = ?1",
        params![media_id],
    )?;

    // Delete associated thumbnails
    tx.execute(
        "DELETE FROM thumbnails WHERE media_id = ?1",
        params![media_id],
    )?;

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
