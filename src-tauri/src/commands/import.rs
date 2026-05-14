use std::sync::Arc;
use tauri::{AppHandle, State, Emitter};
use tauri_plugin_dialog::DialogExt;
use chrono::Local;
use crate::core::error::AppError;
use crate::core::state::LibraryRootState;
use crate::db::pool::DbManager;
use crate::core::constants::IMPORTS_DIRECTORY;
use crate::filesystem::{hash, objects};
use crate::db;
use crate::media::MediaType;
use crate::jobs::{JobType, EnqueueJobRequest};
use crate::core::time::now_ms;
use super::dto::{ImportResultDto, ImportSkippedItemDto, MediaItemDto};

#[tauri::command]
pub async fn import_files(
    app: AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<Option<ImportResultDto>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock
        .as_ref()
        .ok_or(AppError::LibraryRootMissing)
        .inspect_err(AppError::log)?
        .clone();
    drop(root_lock);

    let files = app.dialog().file().blocking_pick_files();
    let Some(file_paths) = files else {
        return Ok(None);
    };

    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let import_dir_rel = format!("{}/{}", IMPORTS_DIRECTORY, date_str);
    let import_dir_abs = root.join(&import_dir_rel);

    std::fs::create_dir_all(&import_dir_abs)
        .map_err(AppError::from)
        .inspect_err(AppError::log)?;

    let conn = db_manager.get_connection(&root).inspect_err(AppError::log)?;
    let mut imported_count: usize = 0;
    let mut skipped_items: Vec<ImportSkippedItemDto> = Vec::new();

    for src_path_buf in file_paths {
        let src_path = src_path_buf
            .as_path()
            .ok_or_else(|| AppError::Other("invalid source path".into()))
            .inspect_err(AppError::log)?;
        let file_name = src_path
            .file_name()
            .ok_or_else(|| AppError::Other("invalid file name".into()))
            .inspect_err(AppError::log)?
            .to_string_lossy()
            .to_string();
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let dest_path_abs = import_dir_abs.join(file_name.as_str());
        let dest_path_rel = format!("{}/{}", import_dir_rel, file_name);

        // 1. Compute hash at original source
        let content_hash = hash::compute_hash(src_path).inspect_err(AppError::log)?;

        let media_type = MediaType::from_extension(ext);
        let now = now_ms();

        // 2. Database Identity
        let media = match db::media::get_by_content_hash(&conn, &content_hash).inspect_err(AppError::log)? {
            Some(m) => {
                let existing_links = db::media_files::list_by_media_id(&conn, m.id).inspect_err(AppError::log)?;
                if !existing_links.is_empty() {
                    let import_prefix = format!("{}/", IMPORTS_DIRECTORY);
                    let mut import_links = db::media_files::list_by_media_in_dir_like(&conn, m.id, &import_prefix).inspect_err(AppError::log)?;
                    import_links.sort_by_key(|link| link.created_at);
                    let original = import_links.first();
                    let original_import_folder = original
                        .and_then(|link| link.dir_path.strip_prefix(&import_prefix).map(|s| s.to_string()));
                    let original_rel_path = original.map(|link| link.rel_path.clone());
                    let existing_link = original.or_else(|| existing_links.first());

                    skipped_items.push(ImportSkippedItemDto {
                        media_id: m.id,
                        content_hash: m.content_hash.clone(),
                        file_name: file_name.clone(),
                        original_import_folder,
                        original_rel_path,
                        existing_dir_path: existing_link.map(|link| link.dir_path.clone()),
                        existing_rel_path: existing_link.map(|link| link.rel_path.clone()),
                    });
                    continue;
                }

                m
            }
            None => db::media::insert_for_hash(&conn, &content_hash, media_type, now).inspect_err(AppError::log)?,
        };

        // 3. Ingest to .objects and projection (hardlink) to Imports
        objects::ingest_and_link(src_path, &root, &content_hash, ext, &dest_path_abs).inspect_err(AppError::log)?;

        // 4. Record the link in the database
        let upsert_result = db::media_files::upsert(&conn, media.id, &dest_path_rel, &dest_path_abs).inspect_err(AppError::log)?;

        // 5. Enqueue jobs
        if upsert_result.is_new || upsert_result.mtime_changed {
            let req = EnqueueJobRequest {
                file_id: upsert_result.file_entry.id,
                media_id: Some(media.id),
                rel_path: dest_path_rel.clone(),
                mtime: upsert_result.file_entry.mtime,
            };
            db::jobs::enqueue(&conn, JobType::Metadata, &req).inspect_err(AppError::log)?;
            db::jobs::enqueue(&conn, JobType::Thumbnail, &req).inspect_err(AppError::log)?;
        }

        imported_count += 1;
    }

    let _ = app.emit("library-changed", ());
    Ok(Some(ImportResultDto {
        folder_name: date_str,
        imported_count,
        skipped_count: skipped_items.len(),
        skipped_items,
    }))
}

#[tauri::command]
pub fn get_import_folders(
    _app: AppHandle,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<Vec<String>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;
    let imports_dir = root.join(IMPORTS_DIRECTORY);

    if !imports_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(imports_dir)?;
    let mut folders = Vec::new();

    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            folders.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    folders.sort_by(|a, b| b.cmp(a)); // Newest first
    Ok(folders)
}

#[tauri::command]
pub fn get_items_in_import_folder(
    _app: AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
    folder_name: String,
) -> Result<Vec<MediaItemDto>, AppError> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or(AppError::LibraryRootMissing).inspect_err(AppError::log)?;

    let conn = db_manager.get_connection(root).inspect_err(AppError::log)?;
    let dir_path = format!("{}/{}", IMPORTS_DIRECTORY, folder_name);

    let items = db::media::get_media_items_in_dir(&conn, &dir_path).inspect_err(AppError::log)?;
    let dtos = items.into_iter().map(MediaItemDto::from).collect();

    Ok(dtos)
}
