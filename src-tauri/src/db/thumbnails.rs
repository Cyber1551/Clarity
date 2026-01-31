use rusqlite::{params, Connection, OptionalExtension};
use crate::core::error::AppResult;
use crate::thumbnails::ThumbnailRow;

/// Inserts or updates a thumbnail for a given content hash.
pub fn upsert(conn: &Connection, thumbnail: ThumbnailRow, now: i64) -> AppResult<()> {
    conn.execute(r#"
        INSERT INTO thumbnails (
            content_hash,
            thumbnail_blob,
            mimetype,
            width,
            height,
            created_at,
            updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
        ON CONFLICT(content_hash) DO UPDATE SET
            thumbnail_blob = excluded.thumbnail_blob,
            mimetype = excluded.mimetype,
            width = excluded.width,
            height = excluded.height,
            updated_at = excluded.updated_at
    "#, params![thumbnail.content_hash, thumbnail.thumbnail_blob, thumbnail.mimetype, thumbnail.width, thumbnail.height, now])?;

    Ok(())
}

/// Retrieves a thumbnail blob and its mimetype by content hash.
pub fn get_thumbnail(conn: &Connection, content_hash: &str) -> AppResult<Option<ThumbnailRow>> {
    let result = conn.query_row(r#"
        SELECT content_hash, thumbnail_blob, mimetype, width, height FROM thumbnails WHERE content_hash = ?1
    "#, params![content_hash], |row| ThumbnailRow::from_row(row)).optional()?;
    Ok(result)
}

/// Retrieves a thumbnail blob by content hash.
pub fn get_blob(conn: &Connection, content_hash: &str) -> AppResult<Option<Vec<u8>>> {
    let blob = conn.query_row(
        "SELECT thumbnail_blob FROM thumbnails WHERE content_hash = ?1",
        params![content_hash],
        |row| row.get(0),
    ).optional()?;
    Ok(blob)
}