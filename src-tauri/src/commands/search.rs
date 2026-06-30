use std::sync::Arc;
use tauri::State;
use crate::core::error::AppError;
use crate::core::state::LibraryRootState;
use crate::db;
use crate::db::pool::DbManager;
use super::dto::{MediaItemDto, SearchQuery};

#[tauri::command]
pub fn search_media(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    query: SearchQuery,
) -> Result<Vec<MediaItemDto>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let items = db::search::search_media(&conn, &query).inspect_err(AppError::log)?;
    Ok(items.into_iter().map(MediaItemDto::from).collect())
}
