use std::sync::Arc;
use tauri::State;
use crate::core::error::AppError;
use crate::core::state::LibraryRootState;
use crate::core::time::now_ms;
use crate::db;
use crate::db::pool::DbManager;
use super::dto::TagDto;

#[tauri::command]
pub fn list_tags(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<Vec<TagDto>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let tags = db::tags::list_all(&conn).inspect_err(AppError::log)?;
    Ok(tags.into_iter().map(TagDto::from).collect())
}

#[tauri::command]
pub fn get_media_tags(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
) -> Result<Vec<TagDto>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let tags = db::tags::list_for_media(&conn, media_id).inspect_err(AppError::log)?;
    Ok(tags.into_iter().map(TagDto::from).collect())
}

/// Assigns a tag (creating it if needed) and marks the item dirty for the By Tag projection.
#[tauri::command]
pub fn add_media_tag(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
    name: String,
) -> Result<TagDto, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let mut conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let tx = conn.transaction().map_err(AppError::from).inspect_err(AppError::log)?;

    let now = now_ms();
    let tag = db::tags::get_or_create(&tx, &name, now).inspect_err(AppError::log)?;
    db::tags::add(&tx, media_id, tag.id, now).inspect_err(AppError::log)?;
    db::media::touch(&tx, media_id, now).inspect_err(AppError::log)?;

    tx.commit().map_err(AppError::from).inspect_err(AppError::log)?;
    Ok(TagDto::from(tag))
}

/// Removes a tag and marks the item dirty for the By Tag projection.
#[tauri::command]
pub fn remove_media_tag(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
    tag_id: i64,
) -> Result<(), AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let mut conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let tx = conn.transaction().map_err(AppError::from).inspect_err(AppError::log)?;

    let now = now_ms();
    db::tags::remove(&tx, media_id, tag_id).inspect_err(AppError::log)?;
    db::media::touch(&tx, media_id, now).inspect_err(AppError::log)?;

    tx.commit().map_err(AppError::from).inspect_err(AppError::log)?;
    Ok(())
}
