use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use rusqlite::Connection;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::database;

/// Global static to store the current database path
pub static DB_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

/// Global static to store the database connection pool
pub static DB_POOL: Lazy<Mutex<Option<Pool<SqliteConnectionManager>>>> = Lazy::new(|| Mutex::new(None));

const DB_NAME: &str = "media_cache.db";
const MAX_POOL_SIZE: u32 = 10;

/// Initialize the database and set up the connection pool
pub async fn initialize_database(folder_path: &str) -> Result<(), String> {
    // Create the database path in the selected folder
    let db_path = PathBuf::from(folder_path).join(DB_NAME);

    // Check if the folder exists
    if !PathBuf::from(folder_path).exists() {
        let error_msg = format!("Folder does not exist: {}", folder_path);
        return Err(error_msg);
    }

    // Create a connection manager
    let manager = SqliteConnectionManager::file(&db_path);
    
    // Create pool with desired size
    let pool = Pool::builder()
        .max_size(MAX_POOL_SIZE)
        .build(manager)
        .map_err(|e| format!("Failed to create connection pool: {}", e))?;
    
    // Store the database path
    let mut db_path_guard = DB_PATH.lock().map_err(|_| "Failed to lock DB_PATH".to_string())?;
    *db_path_guard = Some(db_path);
    
    // Store the connection pool
    let mut pool_guard = DB_POOL.lock().map_err(|_| "Failed to lock DB_POOL".to_string())?;
    *pool_guard = Some(pool);

    // Initialize the tables
    let conn = get_connection()?;
    let _ = database::initialize_tables(&conn)
        .map_err(|e| format!("Failed to initialize tables: {}", e))?;
    
    Ok(())
}

/// Get a connection from the pool
pub fn get_connection() -> Result<r2d2::PooledConnection<SqliteConnectionManager>, String> {
    let pool_guard = DB_POOL.lock().map_err(|_| "Failed to lock DB_POOL".to_string())?;
    
    match &*pool_guard {
        Some(pool) => pool.get()
            .map_err(|e| format!("Failed to get connection from pool: {}", e)),
        None => Err("Database pool not initialized".to_string()),
    }
}






/// Get a rusqlite Connection for compatibility with existing code
/// This opens a new connection using the stored database path
pub fn get_db_connection() -> Result<Connection, String> {
    // Get the database path
    let db_path = get_db_path()?;

    // Open a new connection
    rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))
}

/// Get the database path
pub fn get_db_path() -> Result<PathBuf, String> {
    let db_path_guard = DB_PATH.lock().map_err(|_| "Failed to lock DB_PATH".to_string())?;
    match &*db_path_guard {
        Some(path) => Ok(path.clone()),
        None => Err("Database path not set".to_string()),
    }
}
