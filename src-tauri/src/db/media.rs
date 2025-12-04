use rusqlite::{params, OptionalExtension, Row, Transaction};
use crate::core::error::AppResult;
use crate::filesystem::meta::ProbedMetadata;
use crate::media::{MediaEntry, MediaType};
use crate::jobs::JobStatus;

fn map_row_to_media_entry(row: &Row<'_>) -> rusqlite::Result<MediaEntry> {
    Ok(MediaEntry {
        id: row.get("id")?,
        content_hash: row.get("content_hash")?,
        media_type: row.get("media_type")?,
        width: row.get("width")?,
        height: row.get("height")?,
        duration_ms: row.get("duration_ms")?,
        hash_status: row.get("hash_status")?,
        metadata_status: row.get("metadata_status")?,
        thumbnail_status: row.get("thumbnail_status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

/// Retrieves a media entry by its ID.
pub fn get_by_id(tx: &Transaction, media_id: i64) -> AppResult<Option<MediaEntry>> {
    let existing = tx.query_row(r#"
        SELECT
            id,
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
        FROM media
        WHERE id = ?1
    "#, params![media_id], map_row_to_media_entry).optional()?;

    Ok(existing)
}

/// Retrieves a media entry by its content hash.
pub fn get_by_content_hash(tx: &Transaction, content_hash: &str) -> AppResult<Option<MediaEntry>> {
    let existing = tx.query_row(r#"
        SELECT
            id,
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
        FROM media
        WHERE content_hash = ?1
    "#, params![content_hash], map_row_to_media_entry).optional()?;

    Ok(existing)
}

/// Inserts a new media entry after computing a file's hash.
///
/// Sets hash_status to 'done' and other statuses to 'pending'.
pub fn insert_for_hash(tx: &Transaction, content_hash: &str, media_type: MediaType, now: i64) -> AppResult<MediaEntry> {
    let media_type_str = media_type.to_string();

    tx.execute(r#"
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

    let new_media_id = tx.last_insert_rowid();
    let media_entry = match get_by_id(tx, new_media_id) {
        Ok(Some(media)) => media,
        Ok(None) => panic!("failed to find newly inserted media row"),
        Err(e) => return Err(e),
    };

    Ok(media_entry)
}

/// Updates a media entry with extracted metadata (dimensions, duration).
pub fn update_metadata(tx: &Transaction, media_id: i64, metadata: &ProbedMetadata, now: i64) -> AppResult<()> {
    tx.execute(r#"
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
pub fn mark_metadata_error(tx: &Transaction, media_id: i64, now: i64) -> AppResult<()> {
    tx.execute(r#"
        UPDATE media
        SET metadata_status = 'error',
            updated_at = ?1
        WHERE id = ?2
    "#, params![now, media_id])?;
    Ok(())
}

/// Marks thumbnail generation as complete for a media entry.
pub fn mark_thumbnail_done(tx: &Transaction, media_id: i64, now: i64) -> AppResult<()> {
    tx.execute(r#"
        UPDATE media
        SET thumbnail_status = 'done',
            updated_at = ?1
        WHERE id = ?2
    "#, params![now, media_id])?;
    Ok(())
}

/// Marks thumbnail generation as failed for a media entry.
pub fn mark_thumbnail_error(tx: &Transaction, media_id: i64, now: i64) -> AppResult<()> {
    tx.execute(r#"
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
pub fn get_orphaned_media(tx: &Transaction) -> AppResult<Vec<(i64, String)>> {
    let mut stmt = tx.prepare(r#"
        SELECT m.id, m.content_hash
        FROM media m
        WHERE NOT EXISTS (
            SELECT 1 FROM files f WHERE f.media_id = m.id
        )
    "#)?;

    let orphaned_media = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(orphaned_media)
}

/// Deletes a media row if no files reference it.
///
/// Returns the content_hash if deleted, None otherwise.
pub fn delete_unreferenced_by_id(tx: &Transaction, media_id: i64) -> AppResult<Option<String>> {
    // Atomically delete the media row only if it's still unreferenced.
    // `RETURNING content_hash` gives us the hash if a row was deleted.
    let deleted_hash: Option<String> = tx.query_row(r#"
        DELETE FROM media
        WHERE id = ?1 AND NOT EXISTS (SELECT 1 FROM files WHERE media_id = ?1)
        RETURNING content_hash
    "#, params![media_id], |row| row.get(0)).optional()?;

    Ok(deleted_hash)
}

/// Represents a complete media item with file and thumbnail data for display purposes.
#[derive(Debug, Clone)]
pub struct MediaItemRow {
    pub file_id: i64,
    pub media_id: i64,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub hash_status: JobStatus,
    pub metadata_status: JobStatus,
    pub thumbnail_status: JobStatus,
    pub content_hash: String,
    pub thumbnail_blob: Option<Vec<u8>>,
}

/// Retrieves all media items with their associated file and thumbnail data.
///
/// Joins files, media, and thumbnails tables, ordered by file creation date (newest first).
/// Only returns files that have been hashed and have an associated media entry.
pub fn get_all_with_thumbnails(tx: &Transaction) -> AppResult<Vec<MediaItemRow>> {
    let mut stmt = tx.prepare(r#"
        SELECT
            f.id as file_id,
            f.media_id,
            f.rel_path,
            f.dir_path,
            f.file_name,
            f.ext,
            m.media_type,
            m.width,
            m.height,
            m.duration_ms,
            m.hash_status,
            m.metadata_status,
            m.thumbnail_status,
            m.content_hash,
            t.thumbnail_blob
        FROM files f
        INNER JOIN media m ON f.media_id = m.id
        LEFT JOIN thumbnails t ON m.content_hash = t.content_hash
        ORDER BY f.created_at DESC
    "#)?;

    let rows = stmt.query_map(params![], |row| {
        Ok(MediaItemRow {
            file_id: row.get("file_id")?,
            media_id: row.get("media_id")?,
            rel_path: row.get("rel_path")?,
            dir_path: row.get("dir_path")?,
            file_name: row.get("file_name")?,
            ext: row.get("ext")?,
            media_type: row.get("media_type")?,
            width: row.get("width")?,
            height: row.get("height")?,
            duration_ms: row.get("duration_ms")?,
            hash_status: row.get("hash_status")?,
            metadata_status: row.get("metadata_status")?,
            thumbnail_status: row.get("thumbnail_status")?,
            content_hash: row.get("content_hash")?,
            thumbnail_blob: row.get("thumbnail_blob").ok(),
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(items)
}