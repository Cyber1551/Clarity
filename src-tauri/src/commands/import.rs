use std::sync::Arc;
use tauri::{AppHandle, State, Emitter};
use tauri_plugin_dialog::DialogExt;
use chrono::Local;
use crate::core::state::LibraryRootState;
use crate::db::pool::DbManager;
use crate::core::constants::IMPORTS_DIRECTORY;
use crate::filesystem::{hash, objects};
use crate::db;
use crate::media::MediaType;
use crate::jobs::{JobType, EnqueueJobRequest};
use crate::core::time::now_ms;
use super::dto::MediaItemDto;

#[tauri::command]
pub async fn import_files(
    app: AppHandle,
    db_manager: State<'_, Arc<DbManager>>,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<String, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?.clone();
    drop(root_lock);

    let files = app.dialog().file().blocking_pick_files();
    let Some(file_paths) = files else {
        return Ok("".to_string());
    };

    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let import_dir_rel = format!("{}/{}", IMPORTS_DIRECTORY, date_str);
    let import_dir_abs = root.join(&import_dir_rel);

    std::fs::create_dir_all(&import_dir_abs).map_err(|e| e.to_string())?;

    let conn = db_manager.get_connection(&root).map_err(|e| e.report())?;

    for src_path_buf in file_paths {
        let src_path = src_path_buf.as_path().ok_or("Invalid source path")?;
        let file_name = src_path.file_name().ok_or("Invalid file name")?.to_string_lossy();
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        let dest_path_abs = import_dir_abs.join(file_name.as_ref());
        let dest_path_rel = format!("{}/{}", import_dir_rel, file_name);

        // 1. Compute hash at original source
        let content_hash = hash::compute_hash(src_path).map_err(|e| e.report())?;
        
        let media_type = MediaType::from_extension(ext);
        let now = now_ms();

        // 2. Database Identity
        let media = match db::media::get_by_content_hash(&conn, &content_hash).map_err(|e| e.report())? {
            Some(m) => m,
            None => db::media::insert_for_hash(&conn, &content_hash, media_type, now).map_err(|e| e.report())?,
        };

        // 3. Ingest to .objects and projection (hardlink) to Imports
        objects::ingest_and_link(src_path, &root, &content_hash, ext, &dest_path_abs).map_err(|e| e.report())?;

        // 4. Record the link in the database
        let upsert_result = db::media_links::upsert(&conn, media.id, &dest_path_rel, &dest_path_abs).map_err(|e| e.report())?;

        // 5. Enqueue jobs
        if upsert_result.is_new || upsert_result.mtime_changed {
            let req = EnqueueJobRequest {
                file_id: upsert_result.file_entry.id,
                media_id: Some(media.id),
                rel_path: dest_path_rel.clone(),
                mtime: upsert_result.file_entry.mtime,
            };
            db::jobs::enqueue(&conn, JobType::Metadata, &req).map_err(|e| e.report())?;
            db::jobs::enqueue(&conn, JobType::Thumbnail, &req).map_err(|e| e.report())?;
        }
    }

    let _ = app.emit("library-changed", ());
    Ok(date_str)
}

#[tauri::command]
pub fn get_import_folders(
    _app: AppHandle,
    library_root_state: State<'_, Arc<LibraryRootState>>,
) -> Result<Vec<String>, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;
    let imports_dir = root.join(IMPORTS_DIRECTORY);

    if !imports_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(imports_dir).map_err(|e| e.to_string())?;
    let mut folders = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
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
) -> Result<Vec<MediaItemDto>, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e| e.report())?;
    let dir_path = format!("{}/{}", IMPORTS_DIRECTORY, folder_name);

    let items = db::media::get_media_items_in_dir(&conn, &dir_path).map_err(|e| e.report())?;
    let dtos = items.into_iter().map(MediaItemDto::from).collect();

    Ok(dtos)
}
