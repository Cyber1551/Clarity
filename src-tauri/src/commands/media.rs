use crate::core::constants::{BROKEN_THUMBNAIL, IMPORTS_DIRECTORY, LIBRARY_DIRECTORY};
use crate::core::error::AppError;
use crate::core::time::now_ms;
use crate::db::pool::DbManager;
use crate::core::state::LibraryRootState;
use crate::db;
use crate::media::MediaType;
use std::sync::Arc;
use std::path::Path;
use tauri::{State, Emitter};
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

    let files = db::media_files::list_by_media_id(&tx, media_id).map_err(|e: AppError| e.report())?;
    let objects_abs = objects::find_canonical_objects_file(root, &media.content_hash).map_err(|e: AppError| e.report())?;

    let size_bytes = files.first().map(|f| f.size_bytes).unwrap_or(0);
    let files_dto: Vec<FileDto> = files.into_iter().map(FileDto::from).collect();

    let detail = MediaDetailDto {
        media_id: media.id,
        content_hash: media.content_hash.clone(),
        media_type: media.media_type,
        width: media.width,
        height: media.height,
        duration_ms: media.duration_ms,
        quality_rating: media.quality_rating,
        favorite_rating: media.favorite_rating,
        loved: media.loved,
        size_bytes,
        created_at: media.created_at,
        reviewed_at: media.reviewed_at,
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

#[tauri::command]
pub fn mark_as_reviewed(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;
    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    db::media::mark_reviewed(&conn, media_id, now_ms()).map_err(|e: AppError| e.report())?;
    Ok(())
}

#[tauri::command]
pub fn update_quality_rating(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
    rating: i32,
) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;
    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    db::media::update_quality_rating(&conn, media_id, rating, now_ms()).map_err(|e: AppError| e.report())?;
    Ok(())
}

#[tauri::command]
pub fn update_favorite_rating(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
    rating: i32,
) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;
    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    db::media::update_favorite_rating(&conn, media_id, rating, now_ms()).map_err(|e: AppError| e.report())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_loved(
    _app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
) -> Result<bool, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;
    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    let new_val = db::media::toggle_loved(&conn, media_id, now_ms()).map_err(|e: AppError| e.report())?;
    Ok(new_val)
}

#[tauri::command]
pub fn rename_media_file(
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    file_id: i64,
    new_file_name: String,
) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    let file = db::media_files::get_by_id(&conn, file_id)
        .map_err(|e: AppError| e.report())?
        .ok_or_else(|| "Media file not found".to_string())?;

    let now = now_ms();

    if file.original_file_name.is_none() {
        db::media_files::set_original_file_name(&conn, file_id, &file.file_name, now)
            .map_err(|e: AppError| e.report())?;
    }

    let old_abs = Path::new(root).join(&file.rel_path);
    let new_rel_path = format!("{}/{}.{}", file.dir_path, new_file_name, file.ext);
    let new_abs = Path::new(root).join(&new_rel_path);

    if old_abs != new_abs {
        std::fs::rename(&old_abs, &new_abs).map_err(|e| format!("Failed to rename file: {}", e))?;
    }

    db::media_files::rename(&conn, file_id, &new_file_name, &new_rel_path, now)
        .map_err(|e: AppError| e.report())?;

    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
pub fn review_and_promote(
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    let now = now_ms();

    let media = db::media::get_by_id(&conn, media_id)
        .map_err(|e: AppError| e.report())?
        .ok_or_else(|| "Media not found".to_string())?;

    let import_prefix = format!("{}/", IMPORTS_DIRECTORY);
    let import_links = db::media_files::list_by_media_in_dir_like(&conn, media_id, &import_prefix)
        .map_err(|e: AppError| e.report())?;

    if import_links.is_empty() {
        return Err("No import links found for this media".to_string());
    }

    let representative = &import_links[0];
    let type_folder = match media.media_type {
        MediaType::Image => "Photos",
        MediaType::Video => "Videos",
        MediaType::Unknown => "Other",
    };
    let library_dir_rel = format!("{}/By Type/{}", LIBRARY_DIRECTORY, type_folder);
    let library_rel_path = format!("{}/{}.{}", library_dir_rel, representative.file_name, representative.ext);
    let library_abs_path = Path::new(root).join(&library_rel_path);

    if let Some(parent) = library_abs_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create library dir: {}", e))?;
    }

    let canonical = objects::find_canonical_objects_file(root, &media.content_hash)
        .map_err(|e: AppError| e.report())?;

    if library_abs_path.exists() {
        std::fs::remove_file(&library_abs_path)
            .map_err(|e| format!("Failed to remove existing library link: {}", e))?;
    }
    std::fs::hard_link(&canonical, &library_abs_path)
        .map_err(|e| format!("Failed to create library hardlink: {}", e))?;

    db::media_files::upsert(&conn, media_id, &library_rel_path, &library_abs_path)
        .map_err(|e: AppError| e.report())?;

    for link in &import_links {
        let abs_path = Path::new(root).join(&link.rel_path);
        let _ = std::fs::remove_file(&abs_path);
        db::media_files::delete_by_id(&conn, link.id)
            .map_err(|e: AppError| e.report())?;
    }

    db::media::mark_reviewed(&conn, media_id, now)
        .map_err(|e: AppError| e.report())?;

    let _ = app.emit("library-changed", ());
    Ok(())
}
