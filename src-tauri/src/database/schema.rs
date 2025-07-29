use rusqlite::{Connection, Result};

pub fn initialize_schema(conn: &Connection) -> Result<()> {
    configure_sqlite(conn)?;
    create_tables_if_missing(conn)?;
    Ok(())
}

fn configure_sqlite(conn: &Connection) -> Result<()> {
    let _journal_mode: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get(0))?;
    conn.execute("PRAGMA synchronous = NORMAL;", [])?;
    Ok(())
}

fn create_tables_if_missing(conn: &Connection) -> Result<()> {
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

    Ok(())
}
