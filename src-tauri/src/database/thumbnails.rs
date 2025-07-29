use rusqlite::{params, Connection, Result};
use crate::database::models::Thumbnail;

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