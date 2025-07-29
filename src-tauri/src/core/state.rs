use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::core::constants::{DB_NAME, MAX_POOL_SIZE};
use crate::database;

#[derive(Clone)]
pub struct AppState {
    pub database_pool: Arc<Mutex<Option<Pool<SqliteConnectionManager>>>>
}

impl AppState {
    pub fn get_pool(&self) -> Result<Pool<SqliteConnectionManager>, String> {
        self.database_pool.lock().unwrap().clone().ok_or("Database not initialized".into())
    }
}

/// Initialize the database and set up the connection pool
pub async fn initialize_database(folder_path: &str) -> Result<Pool<SqliteConnectionManager>, String> {
    // Create the database path in the selected folder
    let db_path = PathBuf::from(folder_path).join(DB_NAME);

    // Check if the folder exists
    if !PathBuf::from(folder_path).exists() {
        let error_msg = format!("Folder does not exist: {}", folder_path);
        return Err(error_msg);
    }

    // Create a connection manager
    let manager = SqliteConnectionManager::file(&db_path)
        .with_init(|conn| database::schema::initialize_schema(conn));
    
    // Create pool with desired size
    let pool = Pool::builder()
        .max_size(MAX_POOL_SIZE)
        .build(manager)
        .map_err(|e| format!("Failed to create connection pool: {}", e))?;

    println!("Initialized pool with max size {}", &pool.max_size());

    Ok(pool)
}

/// Get a connection from the pool
pub fn get_connection(pool: &Pool<SqliteConnectionManager>) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, String> {
    pool.get().map_err(|e| format!("Failed to get database connection: {}", e))
}
