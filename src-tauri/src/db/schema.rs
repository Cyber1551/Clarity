use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use crate::core::constants::DB_NAME;
use crate::core::error::{AppResult};

/// Database connection wrapper that auto-configures WAL mode and initializes schema.
pub struct DbConn(pub Connection);

impl std::ops::Deref for DbConn {
    type Target = Connection;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DbConn {
    /// Creates a new database connection and initializes the schema.
    ///
    /// # Parameters
    ///
    /// - `path` - Path to the library root directory
    ///
    /// # Returns
    ///
    /// - `Ok(DbConn)` with a configured connection
    /// - `Err(AppError)` if connection or initialization fails
    ///
    /// # Configuration
    ///
    /// - WAL mode for better concurrency
    /// - Foreign keys enabled
    /// - 5 second busy timeout
    /// - Memory temp storage
    pub fn new<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let db_file = path.as_ref().join(DB_NAME);

        // Read-write, create if missing, single thread
        let conn = Connection::open_with_flags(&db_file,
                                               OpenFlags::SQLITE_OPEN_READ_WRITE
                                                   | OpenFlags::SQLITE_OPEN_CREATE
                                                   | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;

        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        conn.execute_batch(r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA foreign_keys = ON;
        "#)?;

        initialize_schema(&conn)?;
        Ok(Self(conn))
    }
    pub fn transaction(&mut self) -> AppResult<rusqlite::Transaction<'_>> {
        Ok(self.0.transaction()?)
    }
}

fn initialize_schema(conn: &Connection) -> AppResult<()> {
    // `Media` table represents the content itself. Multiple files can point to the same media
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS media (
            id                  INTEGER PRIMARY KEY,
            content_hash        TEXT NOT NULL UNIQUE,
            media_type          TEXT NOT NULL,
            width               INTEGER,
            height              INTEGER,
            duration_ms         INTEGER,
            hash_status         TEXT NOT NULL DEFAULT 'pending',
            metadata_status     TEXT NOT NULL DEFAULT 'pending',
            thumbnail_status    TEXT NOT NULL DEFAULT 'pending',
            created_at          INTEGER NOT NULL,
            updated_at          INTEGER NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_media_content_hash ON media(content_hash);
        CREATE INDEX IF NOT EXISTS idx_media_hash_status ON media(hash_status) WHERE hash_status IN ('pending', 'error');
        CREATE INDEX IF NOT EXISTS idx_media_metadata_status ON media(metadata_status) WHERE metadata_status IN ('pending', 'error');
        CREATE INDEX IF NOT EXISTS idx_media_thumbnail_status ON media(thumbnail_status) WHERE thumbnail_status IN ('pending', 'error');
    "#)?; // AppError::Database

    // `Files` table represent each physical file and hardlink on disk. Does not contain files in the .objects folder
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS files (
            id                  INTEGER PRIMARY KEY,
            media_id            INTEGER REFERENCES media(id),
            rel_path            TEXT NOT NULL UNIQUE,
            dir_path            TEXT NOT NULL,
            file_name           TEXT NOT NULL,
            ext                 TEXT NOT NULL,
            size_bytes          INTEGER NOT NULL,
            mtime               INTEGER NOT NULL,
            last_seen_mtime     INTEGER NOT NULL,
            is_reviewed         INTEGER NOT NULL DEFAULT 0,
            created_at          INTEGER NOT NULL,
            updated_at          INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_files_media_id ON files(media_id);
        CREATE INDEX IF NOT EXISTS idx_files_dir_path ON files(dir_path);
        CREATE INDEX IF NOT EXISTS idx_files_ext ON files(ext);
    "#)?; // AppError::Database

    // `Thumbnails` table hold all thumbnail data for each unique piece of media content.
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS thumbnails (
            content_hash        TEXT PRIMARY KEY REFERENCES media(content_hash) ON DELETE CASCADE,
            thumbnail_blob      BLOB NOT NULL,
            width               INTEGER NOT NULL,
            height              INTEGER NOT NULL,
            created_at          INTEGER NOT NULL,
            updated_at          INTEGER NOT NULL
        );
    "#)?; // AppError::Database

    // `Jobs` table holds all queued and in-progress jobs for hashing, image/video probing, and thumbnail generation
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS jobs (
            id                  INTEGER PRIMARY KEY,
            job_type            TEXT NOT NULL,
            media_id            INTEGER REFERENCES media(id) ON DELETE CASCADE,
            file_id             INTEGER,
            rel_path            TEXT,
            queued_mtime        INTEGER,
            priority            INTEGER NOT NULL DEFAULT 0,
            status              TEXT NOT NULL DEFAULT 'pending',
            attempts            INTEGER NOT NULL DEFAULT 0,
            last_error          TEXT,
            created_at          INTEGER NOT NULL,
            updated_at          INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_jobs_status_priority_created ON jobs(status, priority, created_at);
        CREATE INDEX IF NOT EXISTS idx_jobs_media_id ON jobs(media_id);
        CREATE INDEX IF NOT EXISTS idx_jobs_file_id ON jobs(file_id);
    "#)?; // AppError::Database

    Ok(())
}
