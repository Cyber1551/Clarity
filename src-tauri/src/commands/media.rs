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
    let objects_abs = objects::find_canonical_objects_file(root, &media.content_hash).inspect_err(AppError::log)?;

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

#[tauri::command]
pub fn rename_media_file(
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    file_id: i64,
    new_file_name: String,
) -> Result<(), AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;

    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let file = db::media_files::get_by_id(&conn, file_id)
        .inspect_err(AppError::log)?
        .ok_or_else(|| AppError::NotFound("media file not found".into()))
        .inspect_err(AppError::log)?;

    let now = now_ms();

    if file.original_file_name.is_none() {
        db::media_files::set_original_file_name(&conn, file_id, &file.file_name, now)
            .inspect_err(AppError::log)?;
    }

    let old_abs = Path::new(root).join(&file.rel_path);
    let new_rel_path = format!("{}/{}.{}", file.dir_path, new_file_name, file.ext);
    let new_abs = Path::new(root).join(&new_rel_path);

    if old_abs != new_abs {
        std::fs::rename(&old_abs, &new_abs)
            .map_err(AppError::from)
            .inspect_err(AppError::log)?;
    }

    db::media_files::rename(&conn, file_id, &new_file_name, &new_rel_path, now)
        .inspect_err(AppError::log)?;

    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
pub fn review_and_promote(
    app: tauri::AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    media_id: i64,
) -> Result<(), AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;

    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let now = now_ms();

    let media = db::media::get_by_id(&conn, media_id)
        .inspect_err(AppError::log)?
        .ok_or_else(|| AppError::NotFound("media not found".into()))
        .inspect_err(AppError::log)?;

    let import_prefix = format!("{}/", IMPORTS_DIRECTORY);
    let import_links = db::media_files::list_by_media_in_dir_like(&conn, media_id, &import_prefix)
        .inspect_err(AppError::log)?;

    if import_links.is_empty() {
        let err = AppError::Other("no import links found for this media".into());
        err.log();
        return Err(err);
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
        std::fs::create_dir_all(parent)
            .map_err(AppError::from)
            .inspect_err(AppError::log)?;
    }

    let canonical = objects::find_canonical_objects_file(root, &media.content_hash)
        .inspect_err(AppError::log)?;

    if library_abs_path.exists() {
        std::fs::remove_file(&library_abs_path)
            .map_err(AppError::from)
            .inspect_err(AppError::log)?;
    }
    std::fs::hard_link(&canonical, &library_abs_path)
        .map_err(AppError::from)
        .inspect_err(AppError::log)?;

    db::media_files::upsert(&conn, media_id, &library_rel_path, &library_abs_path)
        .inspect_err(AppError::log)?;

    for link in &import_links {
        let abs_path = Path::new(root).join(&link.rel_path);
        let _ = std::fs::remove_file(&abs_path);
        db::media_files::delete_by_id(&conn, link.id)
            .inspect_err(AppError::log)?;
    }

    db::media::mark_reviewed(&conn, media_id, now)
        .inspect_err(AppError::log)?;

    let _ = app.emit("library-changed", ());
    Ok(())
}
