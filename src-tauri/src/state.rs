use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use rusqlite::Connection;

/// Application state to hold the database connection
pub struct AppState {
    pub db_conn: Mutex<Connection>,
}

/// Global static to store the current database path
pub static DB_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

/// Global static to store the app state
pub static APP_STATE: Lazy<Mutex<Option<AppState>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the database and set up the application state
pub async fn init_database(folder_path: &str) -> Result<(), String> {
    // Create the database path in the selected folder
    let db_path = PathBuf::from(folder_path).join("media_cache.db");

    // Check if the folder exists
    if !PathBuf::from(folder_path).exists() {
        let error_msg = format!("Folder does not exist: {}", folder_path);
        return Err(error_msg);
    }

    // Initialize the database
    let conn = match crate::database::init_db(&db_path) {
        Ok(conn) => conn,
        Err(e) => {
            let error_msg = format!("Failed to initialize database: {}", e);
            return Err(error_msg);
        }
    };

    // Store the database path
    let mut db_path_guard = DB_PATH.lock().map_err(|_| "Failed to lock DB_PATH".to_string())?;
    *db_path_guard = Some(db_path);

    // Create the app state
    let app_state = AppState {
        db_conn: Mutex::new(conn),
    };

    // Store the app state
    let mut app_state_guard = APP_STATE.lock().map_err(|_| "Failed to lock APP_STATE".to_string())?;
    *app_state_guard = Some(app_state);

    Ok(())
}

/// Get the application state
pub fn get_app_state() -> Result<AppState, String> {
    let app_state_guard = APP_STATE.lock().map_err(|_| "Failed to lock APP_STATE".to_string())?;
    match &*app_state_guard {
        Some(_app_state) => {
            // Clone the app state to avoid returning a reference to the guard
            Ok(AppState {
                db_conn: Mutex::new(rusqlite::Connection::open(get_db_path()?)
                    .map_err(|e| format!("Failed to open database: {}", e))?)
            })
        },
        None => Err("Database not initialized".to_string())
    }
}

/// Get the database connection
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
