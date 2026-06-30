use rusqlite::{params, Connection, OptionalExtension};
use crate::core::error::AppResult;
use crate::tags::{slugify, TagRow};

const TAG_COLUMNS: &str = "id, name, slug, created_at, updated_at";

pub fn list_all(conn: &Connection) -> AppResult<Vec<TagRow>> {
    let sql = format!("SELECT {TAG_COLUMNS} FROM tags ORDER BY name COLLATE NOCASE");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], TagRow::from_row)?;
    let mut tags = Vec::new();
    for row in rows { tags.push(row?); }
    Ok(tags)
}

fn get_by_name(conn: &Connection, name: &str) -> AppResult<Option<TagRow>> {
    let sql = format!("SELECT {TAG_COLUMNS} FROM tags WHERE name = ?1 COLLATE NOCASE");
    let tag = conn.query_row(&sql, params![name], TagRow::from_row).optional()?;
    Ok(tag)
}

fn slug_exists(conn: &Connection, slug: &str) -> AppResult<bool> {
    let exists = conn
        .query_row("SELECT 1 FROM tags WHERE slug = ?1", params![slug], |_| Ok(()))
        .optional()?
        .is_some();
    Ok(exists)
}

/// Picks a unique slug for a tag, appending `-2`, `-3`, ... if the base slug collides with a different tag (two distinct names can slugify to the same base).
fn unique_slug(conn: &Connection, base: &str) -> AppResult<String> {
    if !slug_exists(conn, base)? {
        return Ok(base.to_string());
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !slug_exists(conn, &candidate)? {
            return Ok(candidate);
        }
        n += 1;
    }
}

/// Returns the existing tag with the given name (case-insensitive) or creates it.
pub fn get_or_create(conn: &Connection, name: &str, now: i64) -> AppResult<TagRow> {
    let name = name.trim();
    if let Some(existing) = get_by_name(conn, name)? {
        return Ok(existing);
    }

    let slug = unique_slug(conn, &slugify(name))?;
    conn.execute(
        r#"INSERT INTO tags (name, slug, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)"#,
        params![name, slug, now],
    )?;

    let id = conn.last_insert_rowid();
    let sql = format!("SELECT {TAG_COLUMNS} FROM tags WHERE id = ?1");
    let tag = conn.query_row(&sql, params![id], TagRow::from_row)?;
    Ok(tag)
}

pub fn list_for_media(conn: &Connection, media_id: i64) -> AppResult<Vec<TagRow>> {
    let sql = format!(
        r#"
        SELECT {cols}
        FROM tags t
        INNER JOIN media_tags mt ON mt.tag_id = t.id
        WHERE mt.media_id = ?1
        ORDER BY t.name COLLATE NOCASE
    "#,
        cols = TAG_COLUMNS
            .split(", ")
            .map(|c| format!("t.{c}"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![media_id], TagRow::from_row)?;
    let mut tags = Vec::new();
    for row in rows { tags.push(row?); }
    Ok(tags)
}

/// Assigns a tag to a media item (no-op if already assigned).
pub fn add(conn: &Connection, media_id: i64, tag_id: i64, now: i64) -> AppResult<()> {
    conn.execute(
        r#"INSERT OR IGNORE INTO media_tags (media_id, tag_id, created_at) VALUES (?1, ?2, ?3)"#,
        params![media_id, tag_id, now],
    )?;
    Ok(())
}

pub fn remove(conn: &Connection, media_id: i64, tag_id: i64) -> AppResult<()> {
    conn.execute(
        r#"DELETE FROM media_tags WHERE media_id = ?1 AND tag_id = ?2"#,
        params![media_id, tag_id],
    )?;
    Ok(())
}
