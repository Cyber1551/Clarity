use rusqlite::{params, Connection, OptionalExtension};
use crate::core::error::AppResult;
use crate::filesystem::meta::ProbedMetadata;
use crate::media::{MediaItem, MediaRow, MediaType};

/// Retrieves a media row by its ID.
pub fn get_by_id(conn: &Connection, media_id: i64) -> AppResult<Option<MediaRow>> {
    let existing = conn.query_row(r#"
        SELECT
            id,
            content_hash,
            media_type,
            width,
            height,
            duration_ms,
            quality_rating,
            favorite_rating,
            loved,
            hash_status,
            metadata_status,
            thumbnail_status,
            reviewed_at,
            created_at,
            updated_at
        FROM media
        WHERE id = ?1
    "#, params![media_id], MediaRow::from_row).optional()?;

    Ok(existing)
}

/// Retrieves a media row by its content hash.
pub fn get_by_content_hash(conn: &Connection, content_hash: &str) -> AppResult<Option<MediaRow>> {
    let existing = conn.query_row(r#"
        SELECT
            id,
            content_hash,
            media_type,
            width,
            height,
            duration_ms,
            quality_rating,
            favorite_rating,
            loved,
            hash_status,
            metadata_status,
            thumbnail_status,
            reviewed_at,
            created_at,
            updated_at
        FROM media
        WHERE content_hash = ?1
    "#, params![content_hash], MediaRow::from_row).optional()?;

    Ok(existing)
}

/// Marks a media row as reviewed by setting reviewed_at timestamp.
pub fn mark_reviewed(conn: &Connection, media_id: i64, now: i64) -> AppResult<()> {
    conn.execute(
        r#"
        UPDATE media
        SET reviewed_at = ?1,
            updated_at = ?1
        WHERE id = ?2
    "#,
        params![now, media_id],
    )?;
    Ok(())
}

/// Inserts a new media row after computing a file's hash.
///
/// Sets hash_status to 'done' and other statuses to 'pending'.
pub fn insert_for_hash(conn: &Connection, content_hash: &str, media_type: MediaType, now: i64) -> AppResult<MediaRow> {
    let media_type_str = media_type.to_string();

    conn.execute(r#"
        INSERT INTO media (
            content_hash,
            media_type,
            width,
            height,
            duration_ms,
            hash_status,
            metadata_status,
            thumbnail_status,
            created_at,
            updated_at
        )
        VALUES (
            ?1,
            ?2,
            NULL,
            NULL,
            NULL,
            'done',
            'pending',
            'pending',
            ?3,
            ?3
        )
    "#, params![content_hash, media_type_str, now])?;

    let new_media_id = conn.last_insert_rowid();
    let media_row = match get_by_id(conn, new_media_id) {
        Ok(Some(media)) => media,
        Ok(None) => panic!("failed to find newly inserted media row"),
        Err(e) => return Err(e),
    };

    Ok(media_row)
}

/// Updates a media entry with extracted metadata (dimensions, duration).
pub fn update_metadata(conn: &Connection, media_id: i64, metadata: &ProbedMetadata, now: i64) -> AppResult<()> {
    conn.execute(r#"
        UPDATE media
        SET
            width = ?1,
            height = ?2,
            duration_ms = ?3,
            metadata_status = 'done',
            updated_at = ?4
        WHERE id = ?5
    "#, params![metadata.width, metadata.height, metadata.duration_ms, now, media_id])?;

    Ok(())
}

/// Marks metadata extraction as failed for a media entry.
pub fn mark_metadata_error(conn: &Connection, media_id: i64, now: i64) -> AppResult<()> {
    conn.execute(r#"
        UPDATE media
        SET metadata_status = 'error',
            updated_at = ?1
        WHERE id = ?2
    "#, params![now, media_id])?;
    Ok(())
}

/// Marks thumbnail generation as complete for a media entry.
pub fn mark_thumbnail_done(conn: &Connection, media_id: i64, now: i64) -> AppResult<()> {
    conn.execute(r#"
        UPDATE media
        SET thumbnail_status = 'done',
            updated_at = ?1
        WHERE id = ?2
    "#, params![now, media_id])?;
    Ok(())
}

pub fn update_quality_rating(conn: &Connection, media_id: i64, rating: i32, now: i64) -> AppResult<()> {
    conn.execute(
        r#"UPDATE media SET quality_rating = ?1, updated_at = ?2 WHERE id = ?3"#,
        params![rating, now, media_id],
    )?;
    Ok(())
}

pub fn update_favorite_rating(conn: &Connection, media_id: i64, rating: i32, now: i64) -> AppResult<()> {
    conn.execute(
        r#"UPDATE media SET favorite_rating = ?1, updated_at = ?2 WHERE id = ?3"#,
        params![rating, now, media_id],
    )?;
    Ok(())
}

pub fn toggle_loved(conn: &Connection, media_id: i64, now: i64) -> AppResult<bool> {
    let current: bool = conn.query_row(
        "SELECT loved FROM media WHERE id = ?1",
        params![media_id],
        |row| row.get(0),
    )?;
    let new_val = !current;
    conn.execute(
        r#"UPDATE media SET loved = ?1, updated_at = ?2 WHERE id = ?3"#,
        params![new_val, now, media_id],
    )?;
    Ok(new_val)
}

/// Marks thumbnail generation as failed for a media entry.
pub fn mark_thumbnail_error(conn: &Connection, media_id: i64, now: i64) -> AppResult<()> {
    conn.execute(r#"
        UPDATE media
        SET thumbnail_status = 'error',
            updated_at = ?1
        WHERE id = ?2
    "#, params![now, media_id])?;
    Ok(())
}

/// Retrieves all orphaned media entries (media with no file references).
///
/// Returns a list of (media_id, content_hash) tuples.
pub fn get_orphaned_media(conn: &Connection) -> AppResult<Vec<(i64, String)>> {
    let mut stmt = conn.prepare(r#"
        SELECT m.id, m.content_hash
        FROM media m
        WHERE NOT EXISTS (
            SELECT 1 FROM media_files f WHERE f.media_id = m.id
        )
    "#)?;

    let orphaned_media = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(orphaned_media)
}

/// Deletes a media row if no media_files reference it.
///
/// Returns the content_hash if deleted, None otherwise.
pub fn delete_unreferenced_by_id(conn: &Connection, media_id: i64) -> AppResult<Option<String>> {
    // Atomically delete the media row only if it's still unreferenced.
    // `RETURNING content_hash` gives us the hash if a row was deleted.
    let deleted_hash: Option<String> = conn.query_row(r#"
        DELETE FROM media
        WHERE id = ?1 AND NOT EXISTS (SELECT 1 FROM media_files WHERE media_id = ?1)
        RETURNING content_hash
    "#, params![media_id], |row| row.get(0)).optional()?;

    Ok(deleted_hash)
}

// ===================== Media Items (Media + Path) =====================

pub fn get_media_items(conn: &Connection) -> AppResult<Vec<MediaItem>> {
    let mut stmt = conn.prepare(r#"
        SELECT
            m.id,
            m.content_hash,
            m.media_type,
            m.width,
            m.height,
            m.duration_ms,
            m.quality_rating,
            m.favorite_rating,
            m.loved,
            m.hash_status,
            m.metadata_status,
            m.thumbnail_status,
            m.reviewed_at,
            m.created_at,
            m.updated_at,
            rf.rel_path,
            rf.dir_path,
            rf.file_name,
            rf.ext
        FROM media m
        INNER JOIN media_files rf ON rf.id = ( 
            SELECT id FROM media_files 
            WHERE media_id = m.id AND dir_path LIKE 'Library%' 
            LIMIT 1 
        )
        ORDER BY m.created_at DESC
    "#)?;

    let rows = stmt.query_map(params![], |row| {
        Ok(MediaItem {
            media: MediaRow::from_row(row)?,
            rel_path: row.get("rel_path")?,
            dir_path: row.get("dir_path")?,
            file_name: row.get("file_name")?,
            ext: row.get("ext")?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows { items.push(row?); }
    Ok(items)
}

pub fn get_media_items_in_dir(conn: &Connection, dir_path: &str) -> AppResult<Vec<MediaItem>> {
    let mut stmt = conn.prepare(r#"
        SELECT
            m.id,
            m.content_hash,
            m.media_type,
            m.width,
            m.height,
            m.duration_ms,
            m.quality_rating,
            m.favorite_rating,
            m.loved,
            m.hash_status,
            m.metadata_status,
            m.thumbnail_status,
            m.reviewed_at,
            m.created_at,
            m.updated_at,
            rf.rel_path,
            rf.dir_path,
            rf.file_name,
            rf.ext
        FROM media m
        INNER JOIN media_files rf ON rf.media_id = m.id
        WHERE rf.dir_path = ?1
        ORDER BY m.created_at DESC
    "#)?;

    let rows = stmt.query_map(params![dir_path], |row| {
        Ok(MediaItem {
            media: MediaRow::from_row(row)?,
            rel_path: row.get("rel_path")?,
            dir_path: row.get("dir_path")?,
            file_name: row.get("file_name")?,
            ext: row.get("ext")?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows { items.push(row?); }
    Ok(items)
}

pub fn get_media_item_by_rel_path(conn: &Connection, rel_path: &str) -> AppResult<Option<MediaItem>> {
    let mut stmt = conn.prepare(r#"
        SELECT
            m.id,
            m.content_hash,
            m.media_type,
            m.width,
            m.height,
            m.duration_ms,
            m.quality_rating,
            m.favorite_rating,
            m.loved,
            m.hash_status,
            m.metadata_status,
            m.thumbnail_status,
            m.reviewed_at,
            m.created_at,
            m.updated_at,
            rf.rel_path,
            rf.dir_path,
            rf.file_name,
            rf.ext
        FROM media m
        INNER JOIN media_files rf ON rf.media_id = m.id
        WHERE rf.rel_path = ?1
        LIMIT 1
    "#)?;

    let item = stmt.query_row(params![rel_path], |row| {
        Ok(MediaItem {
            media: MediaRow::from_row(row)?,
            rel_path: row.get("rel_path")?,
            dir_path: row.get("dir_path")?,
            file_name: row.get("file_name")?,
            ext: row.get("ext")?,
        })
    }).optional()?;

    Ok(item)
}
