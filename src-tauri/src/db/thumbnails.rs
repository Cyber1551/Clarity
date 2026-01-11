use rusqlite::{params, Connection, OptionalExtension};
use crate::core::error::AppResult;

/// Inserts or updates a thumbnail for a given content hash.
pub fn upsert(conn: &Connection, content_hash: &str, thumb_blob: &[u8], width: i64, height: i64, now: i64) -> AppResult<()> {
    conn.execute(r#"
        INSERT INTO thumbnails (
            content_hash,
            thumbnail_blob,
            width,
            height,
            created_at,
            updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?5)
        ON CONFLICT(content_hash) DO UPDATE SET
            thumbnail_blob = excluded.thumbnail_blob,
            width = excluded.width,
            height = excluded.height,
            updated_at = excluded.updated_at
    "#, params![content_hash, thumb_blob, width, height, now])?;

    Ok(())
}

/// Retrieves a thumbnail blob by content hash.
pub fn get_blob(conn: &rusqlite::Connection, content_hash: &str) -> AppResult<Option<Vec<u8>>> {
    let blob = conn.query_row(
        "SELECT thumbnail_blob FROM thumbnails WHERE content_hash = ?1",
        params![content_hash],
        |row| row.get(0),
    ).optional()?;
    Ok(blob)
}