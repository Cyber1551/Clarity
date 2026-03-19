use std::path::{Path, PathBuf};
use std::sync::Mutex;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use crate::core::error::{AppResult, AppError};

pub type DbPool = Pool<SqliteConnectionManager>;

pub struct DbManager {
    pool: Mutex<Option<(PathBuf, DbPool)>>,
}

impl DbManager {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(None),
        }
    }

    pub fn get_pool(&self, library_root: &Path) -> AppResult<DbPool> {
        let mut lock = self.pool.lock().unwrap();
        
        if let Some((ref path, ref pool)) = *lock {
            if path == library_root {
                return Ok(pool.clone());
            }
        }

        // Initialize a new db pool
        let db_file = library_root.join(crate::core::constants::DB_NAME);
        let manager = SqliteConnectionManager::file(db_file)
            .with_init(|conn| {
                // Configure the connection
                conn.pragma_update(None, "journal_mode", &"WAL")?;
                conn.pragma_update(None, "synchronous", &"NORMAL")?;
                conn.pragma_update(None, "temp_store", &"MEMORY")?;
                conn.pragma_update(None, "foreign_keys", &"ON")?;
                Ok(())
            });

        let pool = Pool::builder()
            .max_size((num_cpus::get() as u32 * 2).max(20))
            .build(manager)
            .map_err(|e| AppError::Other(e.to_string()))?;

        // Initialize schema and migrations EXACTLY ONCE
        {
            let mut conn = pool.get().map_err(|e| AppError::Other(e.to_string()))?;
            crate::db::schema::initialize_schema(&mut conn)?;
        }

        *lock = Some((library_root.to_path_buf(), pool.clone()));
        Ok(pool)
    }

    pub fn get_connection(&self, library_root: &Path) -> AppResult<PooledConnection<SqliteConnectionManager>> {
        let pool = self.get_pool(library_root)?;
        pool.get().map_err(|e| AppError::Other(e.to_string()))
    }
}
