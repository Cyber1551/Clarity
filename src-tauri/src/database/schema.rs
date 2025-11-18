use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use crate::core::constants::DB_NAME;
use crate::errors::{AppResult};

pub struct DbConn(pub Connection);

impl DbConn {
    pub fn new(path: &Path) -> AppResult<Self> {
        let db_file = path.join(DB_NAME);

        // Read-write, create if missing, single thread
        let conn = Connection::open_with_flags(&db_file,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;

        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA foreign_keys = ON;
        ")?;

        initialize_schema(&conn)?;

        Ok(DbConn(conn))
    }
}

fn initialize_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS media_files (
            id           INTEGER PRIMARY KEY,
            path         TEXT NOT NULL UNIQUE,
            rel_path     TEXT NOT NULL,
            dir_path     TEXT NOT NULL,
            size_bytes   INTEGER NOT NULL,
            mtime_secs   INTEGER NOT NULL,
            content_hash TEXT,
            media_type   TEXT NOT NULL,
            ext          TEXT NOT NULL,
            width        INTEGER,
            height       INTEGER,
            duration_ms  INTEGER,
            created_at     INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS thumbnails (
            content_hash TEXT PRIMARY KEY,
            thumb_blob   BLOB NOT NULL,
            width        INTEGER NOT NULL,
            height       INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_media_dir ON media_files(dir_path);
        CREATE INDEX IF NOT EXISTS idx_media_hash ON media_files(content_hash);
    ")?; // AppError::Database

    Ok(())
}
