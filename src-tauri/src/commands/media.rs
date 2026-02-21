use crate::core::constants::{BROKEN_THUMBNAIL};
use crate::core::error::AppError;
use crate::db::pool::DbManager;
use crate::core::state::LibraryRootState;
use crate::db;
use std::sync::Arc;
use tauri::{State};
use crate::filesystem::objects;
use super::dto::*;

// ============ Viewer/Tags DTOs and Commands ============

// ============ Media Items (Gallery) ============

#[tauri::command]
pub fn get_media_items(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<Vec<MediaItemDto>, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;

    let items = db::media::get_media_items(&conn).map_err(|e: AppError| e.report())?;
    let dtos: Vec<MediaItemDto> = items.into_iter().map(MediaItemDto::from).collect();

    Ok(dtos)
}

#[tauri::command]
pub fn get_media_detail(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, media_id: i64) -> Result<MediaDetailDto, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let mut conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    let tx = conn.transaction().map_err(|e: rusqlite::Error| AppError::Database(e).report())?;

    let media = db::media::get_by_id(&tx, media_id).map_err(|e: AppError| e.report())?
        .ok_or_else(|| AppError::NotFound("media not found".into()).report())?;

    let files = db::media_links::list_by_media_id(&tx, media_id).map_err(|e: AppError| e.report())?;
    let objects_abs = objects::find_canonical_objects_file(root, &media.content_hash).map_err(|e: AppError| e.report())?;

    let files_dto: Vec<FileDto> = files.into_iter().map(FileDto::from).collect();

    let detail = MediaDetailDto {
        media_id: media.id,
        content_hash: media.content_hash,
        media_type: media.media_type,
        width: media.width,
        height: media.height,
        duration_ms: media.duration_ms,
        quality_rating: media.quality_rating,
        favorite_rating: media.favorite_rating,
        loved: media.loved,
        files: files_dto,
        canonical_path: objects_abs.to_string_lossy().to_string(),
    };

    tx.commit().map_err(|e| e.to_string())?;
    Ok(detail)
}

#[tauri::command]
pub fn get_thumbnail(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, hash: String) -> Result<ThumbnailDto, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    let thumb = db::thumbnails::get_thumbnail(&conn, &hash).map_err(|e: AppError| e.report())?;
    
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
) -> Result<Option<MediaItemDto>, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    let item = db::media::get_media_item_by_rel_path(&conn, &rel_path).map_err(|e: AppError| e.report())?;

    Ok(item.map(MediaItemDto::from))
}
