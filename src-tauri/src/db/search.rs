use rusqlite::types::Value;
use rusqlite::Connection;
use crate::commands::dto::SearchQuery;
use crate::core::error::AppResult;
use crate::media::{MediaItem, MediaRow};
use super::media::media_select_list;

const DEFAULT_LIMIT: i64 = 200;
const MAX_LIMIT: i64 = 1000;

/// Searches media by optional FTS title match plus structured filters. 
/// Results are `MediaItem`s identical to the gallery (same representative-file LEFT JOIN), so the viewer/grid can reuse them.
pub fn search_media(conn: &Connection, query: &SearchQuery) -> AppResult<Vec<MediaItem>> {
    let match_expr = query
        .text
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(build_match_expr);

    let mut sql = format!("SELECT {cols},", cols = media_select_list("m."));
    sql.push_str(" rf.rel_path, rf.dir_path, rf.file_name, rf.ext FROM media m ");

    if match_expr.is_some() {
        sql.push_str("JOIN media_fts f ON f.rowid = m.id ");
    }

    sql.push_str(
        "LEFT JOIN media_files rf ON rf.id = (\
         SELECT id FROM media_files WHERE media_id = m.id \
         ORDER BY (dir_path LIKE 'Library%') DESC, id LIMIT 1) ",
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();

    if let Some(expr) = &match_expr {
        conditions.push("media_fts MATCH ?".to_string());
        params.push(Value::Text(expr.clone()));
    }
    if let Some(media_type) = query.media_type {
        conditions.push("m.media_type = ?".to_string());
        params.push(Value::Text(media_type.to_string()));
    }
    if let Some(quality) = query.quality {
        conditions.push("m.quality_rating >= ?".to_string());
        params.push(Value::Integer(quality.into()));
    }
    if let Some(favorite) = query.favorite {
        conditions.push("m.favorite_rating >= ?".to_string());
        params.push(Value::Integer(favorite.into()));
    }
    if let Some(loved) = query.loved {
        conditions.push("m.loved = ?".to_string());
        params.push(Value::Integer(loved.into()));
    }
    if let Some(reviewed) = query.reviewed {
        conditions.push(
            if reviewed { "m.reviewed_at IS NOT NULL" } else { "m.reviewed_at IS NULL" }.to_string(),
        );
    }
    for slug in &query.tags {
        conditions.push(
            "EXISTS (SELECT 1 FROM media_tags mt JOIN tags t ON t.id = mt.tag_id \
             WHERE mt.media_id = m.id AND t.slug = ?)"
                .to_string(),
        );
        params.push(Value::Text(slug.clone()));
    }

    if !conditions.is_empty() {
        sql.push_str("WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push(' ');
    }

    // bm25 ranks best matches first (lowest score); fall back to recency for filter-only searches.
    sql.push_str(if match_expr.is_some() {
        "ORDER BY bm25(media_fts) "
    } else {
        "ORDER BY m.created_at DESC "
    });

    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = query.offset.unwrap_or(0).max(0);
    sql.push_str("LIMIT ? OFFSET ?");
    params.push(Value::Integer(limit));
    params.push(Value::Integer(offset));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
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

/// Quotes each whitespace term (doubling internal `"`) and appends `*` for prefix matching, joined by spaces (implicit AND).
/// Quoting makes arbitrary user input safe from FTS5 syntax errors.
fn build_match_expr(text: &str) -> Option<String> {
    let terms: Vec<String> = text
        .split_whitespace()
        .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
        .collect();
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}
