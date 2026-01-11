use rusqlite::{params, Connection, OptionalExtension, Row};
use crate::core::error::AppResult;
use crate::core::time::now_ms;

#[derive(Debug, Clone)]
pub struct TagRow {
    pub id: i64,
    pub name: String,
}

fn map_row_to_tag(row: &Row<'_>) -> rusqlite::Result<TagRow> {
    Ok(TagRow {
        id: row.get("id")?,
        name: row.get("name")?,
    })
}

pub fn list_all(conn: &Connection) -> AppResult<Vec<TagRow>> {
    let mut stmt = conn.prepare(r#"
        SELECT id, name
        FROM tags
        ORDER BY name COLLATE NOCASE ASC
    "#)?;

    let rows = stmt.query_map([], map_row_to_tag)?;
    let mut tags = Vec::new();
    for r in rows {
        tags.push(r?);
    }
    Ok(tags)
}

pub fn get_by_name(conn: &Connection, name: &str) -> AppResult<Option<TagRow>> {
    let existing = conn
        .query_row(
            r#"
            SELECT id, name
            FROM tags
            WHERE name = ?1
        "#,
            params![name],
            map_row_to_tag,
        )
        .optional()?;
    Ok(existing)
}

pub fn create(conn: &Connection, name: &str) -> AppResult<TagRow> {
    let now = now_ms();
    conn.execute(
        r#"
        INSERT INTO tags (name, created_at, updated_at)
        VALUES (?1, ?2, ?2)
    "#,
        params![name, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(TagRow { id, name: name.to_string() })
}

pub fn get_or_create(conn: &Connection, name: &str) -> AppResult<TagRow> {
    if let Some(t) = get_by_name(conn, name)? { return Ok(t); }
    create(conn, name)
}

pub fn list_for_media(conn: &Connection, media_id: i64) -> AppResult<Vec<TagRow>> {
    let mut stmt = conn.prepare(r#"
        SELECT t.id, t.name
        FROM media_tags mt
        JOIN tags t ON t.id = mt.tag_id
        WHERE mt.media_id = ?1
        ORDER BY t.name COLLATE NOCASE ASC
    "#)?;
    let rows = stmt.query_map(params![media_id], map_row_to_tag)?;
    let mut tags = Vec::new();
    for r in rows { tags.push(r?); }
    Ok(tags)
}

pub fn add_tag_to_media(conn: &Connection, media_id: i64, tag_id: i64) -> AppResult<bool> {
    let now = now_ms();
    let changed = conn.execute(
        r#"
        INSERT OR IGNORE INTO media_tags (media_id, tag_id, created_at)
        VALUES (?1, ?2, ?3)
    "#,
        params![media_id, tag_id, now],
    )?;
    Ok(changed > 0)
}

pub fn remove_tag_from_media(conn: &Connection, media_id: i64, tag_id: i64) -> AppResult<bool> {
    let changed = conn.execute(
        r#"
        DELETE FROM media_tags
        WHERE media_id = ?1 AND tag_id = ?2
    "#,
        params![media_id, tag_id],
    )?;
    Ok(changed > 0)
}
