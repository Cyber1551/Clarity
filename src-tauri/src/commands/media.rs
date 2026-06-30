use crate::core::constants::BROKEN_THUMBNAIL;
use crate::core::error::AppError;
use crate::core::time::now_ms;
use crate::db::pool::DbManager;
use crate::core::state::LibraryRootState;
use crate::db;
use std::sync::Arc;
use tauri::State;
use crate::filesystem::objects;
use super::dto::*;

#[tauri::command]
pub fn get_media_items(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<Vec<MediaItemDto>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;

    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let items = db::media::get_media_items(&conn).inspect_err(AppError::log)?;
    let dtos: Vec<MediaItemDto> = items.into_iter().map(MediaItemDto::from).collect();

    Ok(dtos)
}

#[tauri::command]
pub fn get_media_detail(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, media_id: i64) -> Result<MediaDetailDto, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;

    let mut conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let tx = conn.transaction().map_err(AppError::from).inspect_err(AppError::log)?;

    let media = db::media::get_by_id(&tx, media_id).inspect_err(AppError::log)?
        .ok_or_else(|| AppError::NotFound("media not found".into()))
        .inspect_err(AppError::log)?;

    let files = db::media_files::list_by_media_id(&tx, media_id).inspect_err(AppError::log)?;
    let tags = db::tags::list_for_media(&tx, media_id).inspect_err(AppError::log)?;
    let objects_abs = objects::find_canonical_objects_file(root, &media.content_hash).inspect_err(AppError::log)?;

    let size_bytes = files.first().map(|f| f.size_bytes).unwrap_or(0);
    let files_dto: Vec<FileDto> = files.into_iter().map(FileDto::from).collect();
    let tags_dto: Vec<TagDto> = tags.into_iter().map(TagDto::from).collect();

    let detail = MediaDetailDto {
        media_id: media.id,
        content_hash: media.content_hash.clone(),
        media_type: media.media_type,
        display_name: media.display_name.clone(),
        original_file_name: media.original_file_name.clone(),
        width: media.width,
        height: media.height,
        duration_ms: media.duration_ms,
        quality_rating: media.quality_rating,
        favorite_rating: media.favorite_rating,
        loved: media.loved,
        size_bytes,
        created_at: media.created_at,
        reviewed_at: media.reviewed_at,
        tags: tags_dto,
        files: files_dto,
        canonical_path: objects_abs.to_string_lossy().to_string(),
    };

    tx.commit().map_err(AppError::from).inspect_err(AppError::log)?;
    Ok(detail)
}

#[tauri::command]
pub fn get_thumbnail(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, hash: String) -> Result<ThumbnailDto, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;

    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let thumb = db::thumbnails::get_thumbnail(&conn, &hash).inspect_err(AppError::log)?;

    match thumb {
        Some(row) => Ok(ThumbnailDto::from(row)),
        None => {
            // Return broken thumbnail if not found
            Ok(ThumbnailDto {
                blob: BROKEN_THUMBNAIL.to_vec(),
                mimetype: "image/webp".to_string(),
            })
        }
    }
}

#[tauri::command]
pub fn get_media_item_by_rel_path(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    rel_path: String,
) -> Result<Option<MediaItemDto>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;

    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let item = db::media::get_media_item_by_rel_path(&conn, &rel_path).inspect_err(AppError::log)?;

    Ok(item.map(MediaItemDto::from))
}

/// Flips the reviewed flag only; the move from Imports into the Library tree happens on next sync.
#[tauri::command]
pub fn mark_as_reviewed(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
) -> Result<(), AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    db::media::mark_reviewed(&conn, media_id, now_ms()).inspect_err(AppError::log)?;
    Ok(())
}

#[tauri::command]
pub fn update_quality_rating(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
    rating: i32,
) -> Result<(), AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    db::media::update_quality_rating(&conn, media_id, rating, now_ms()).inspect_err(AppError::log)?;
    Ok(())
}

#[tauri::command]
pub fn update_favorite_rating(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
    rating: i32,
) -> Result<(), AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    db::media::update_favorite_rating(&conn, media_id, rating, now_ms()).inspect_err(AppError::log)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_loved(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
) -> Result<bool, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let new_val = db::media::toggle_loved(&conn, media_id, now_ms()).inspect_err(AppError::log)?;
    Ok(new_val)
}

/// Logical rename via display_name; the projected hardlinks are renamed on the next sync.
#[tauri::command]
pub fn rename_media(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
    new_name: String,
) -> Result<(), AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;

    let new_name = new_name.trim();
    if new_name.is_empty() {
        let err = AppError::Other("name cannot be empty".into());
        err.log();
        return Err(err);
    }

    db::media::update_display_name(&conn, media_id, new_name, now_ms()).inspect_err(AppError::log)?;
    Ok(())
}
