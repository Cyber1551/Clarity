use serde::{Serialize, Deserialize};
use crate::core::constants::{SORTED_DIRECTORY, UNSORTED_DIRECTORY};
use crate::core::error::AppError;
use crate::db::pool::DbManager;
use crate::core::state::LibraryRootState;
use crate::db;
use crate::jobs::JobStatus;
use crate::media::MediaType;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use tauri::{Emitter, State};
use crate::filesystem::objects;
use crate::core::time::now_ms;

/// Data transfer object for media items sent to the frontend.
///
/// Combines file metadata and media information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDto {
    pub media_id: i64,
    pub file_id: i64,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub hash_status: JobStatus,
    pub metadata_status: JobStatus,
    pub thumbnail_status: JobStatus,
    pub content_hash: String,
}

/// Retrieves all media items from the library.
#[tauri::command]
pub fn get_all_media(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<Vec<MediaItemDto>, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;
    
    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;

    // Delegate to database layer
    let rows = db::media::get_all_with_thumbnails(&conn).map_err(|e: AppError| e.report())?;

    let items: Vec<MediaItemDto> = rows
        .into_iter()
        .map(MediaItemDto::from_db_row)
        .collect();

    Ok(items)
}

impl MediaItemDto {
    fn from_db_row(row: db::media::MediaItemRow) -> Self {
        Self {
            media_id: row.media_id,
            file_id: row.file_id,
            rel_path: row.rel_path,
            dir_path: row.dir_path,
            file_name: row.file_name,
            ext: row.ext,
            media_type: row.media_type,
            width: row.width,
            height: row.height,
            duration_ms: row.duration_ms,
            hash_status: row.hash_status,
            metadata_status: row.metadata_status,
            thumbnail_status: row.thumbnail_status,
            content_hash: row.content_hash,
        }
    }
}

// ============ Viewer/Tags DTOs and Commands ============

// ============ Media-level feed (one row per media) ============

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFeedItemDto {
    pub media_id: i64,
    pub rel_path: Option<String>,
    pub dir_path: Option<String>,
    pub file_name: Option<String>,
    pub ext: Option<String>,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub hash_status: JobStatus,
    pub metadata_status: JobStatus,
    pub thumbnail_status: JobStatus,
    pub content_hash: String,
    pub reviewed_at: Option<i64>,
    pub tags: Vec<TagDto>,
}

impl MediaFeedItemDto {
    fn from_db_row(row: db::media::MediaFeedRow) -> Self {
        Self {
            media_id: row.media_id,
            rel_path: row.rel_path,
            dir_path: row.dir_path,
            file_name: row.file_name,
            ext: row.ext,
            media_type: row.media_type,
            width: row.width,
            height: row.height,
            duration_ms: row.duration_ms,
            hash_status: row.hash_status,
            metadata_status: row.metadata_status,
            thumbnail_status: row.thumbnail_status,
            content_hash: row.content_hash,
            reviewed_at: row.reviewed_at,
            tags: row.tags,
        }
    }
}

#[tauri::command]
pub fn get_media_feed(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<Vec<MediaFeedItemDto>, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;

    let rows = db::media::get_media_feed(&conn).map_err(|e: AppError| e.report())?;
    let items: Vec<MediaFeedItemDto> = rows.into_iter().map(MediaFeedItemDto::from_db_row).collect();

    Ok(items)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDto {
    pub id: i64,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDto { pub id: i64, pub name: String }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDetailDto {
    pub media_id: i64,
    pub content_hash: String,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub files: Vec<FileDto>,
    pub tags: Vec<TagDto>,
}

#[tauri::command]
pub fn get_media_detail(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, media_id: i64) -> Result<MediaDetailDto, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let mut conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;
    let tx = conn.transaction().map_err(|e: rusqlite::Error| AppError::Database(e).report())?;

    let media = db::media::get_by_id(&tx, media_id).map_err(|e: AppError| e.report())?
        .ok_or_else(|| AppError::NotFound("media not found".into()).report())?;

    let files = db::files::list_by_media_id(&tx, media_id).map_err(|e: AppError| e.report())?;
    let tags = db::tags::list_for_media(&tx, media_id).map_err(|e: AppError| e.report())?;

    let files_dto = files.into_iter().map(|f| FileDto {
        id: f.id,
        rel_path: f.rel_path,
        dir_path: f.dir_path,
        file_name: f.file_name,
        ext: f.ext,
    }).collect();

    let tags_dto = tags.into_iter().map(|t| TagDto { id: t.id, name: t.name }).collect();

    let detail = MediaDetailDto {
        media_id: media.id,
        content_hash: media.content_hash,
        media_type: media.media_type,
        width: media.width,
        height: media.height,
        duration_ms: media.duration_ms,
        files: files_dto,
        tags: tags_dto,
    };

    tx.commit().map_err(|e| e.to_string())?;
    Ok(detail)
}

#[tauri::command]
pub fn list_tags(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>) -> Result<Vec<TagDto>, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;

    let tags = db::tags::list_all(&conn).map_err(|e: AppError| e.report())?;
    let dtos = tags.into_iter().map(|t| TagDto { id: t.id, name: t.name }).collect();
    Ok(dtos)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest { pub name: String }

#[tauri::command]
pub fn create_tag(_app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, req: CreateTagRequest) -> Result<TagDto, String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?;

    let conn = db_manager.get_connection(root).map_err(|e: AppError| e.report())?;

    let tag = db::tags::get_or_create(&conn, &req.name).map_err(|e: AppError| e.report())?;
    Ok(TagDto { id: tag.id, name: tag.name })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagMediaRequest { pub media_id: i64, pub tag_id: i64 }

#[tauri::command]
pub fn tag_media(app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, req: TagMediaRequest) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?.clone();
    drop(root_lock);

    let mut conn = db_manager.get_connection(&root).map_err(|e: AppError| e.report())?;
    let tx = conn.transaction().map_err(|e: rusqlite::Error| AppError::Database(e).report())?;

    // DB relation
    let changed = db::tags::add_tag_to_media(&tx, req.media_id, req.tag_id).map_err(|e: AppError| e.report())?;

    // Ensure a single hardlink in Sorted/[Tag]/ for this media (one link per folder per media).
    if changed {
        // get tag name and media entry
        let tag = db::tags::list_all(&tx).map_err(|e: AppError| e.report())?
            .into_iter().find(|t| t.id == req.tag_id)
            .ok_or_else(|| AppError::NotFound("tag not found".into()).report())?;
        let media = db::media::get_by_id(&tx, req.media_id).map_err(|e: AppError| e.report())?
            .ok_or_else(|| AppError::NotFound("media not found".into()).report())?;

        let files = db::files::list_by_media_id(&tx, req.media_id).map_err(|e: AppError| e.report())?;
        let objects_abs = objects::find_canonical_objects_file(&root, &media.content_hash).map_err(|e: AppError| e.report())?;

        let mut tag_dir = PathBuf::from(SORTED_DIRECTORY);
        tag_dir.push(&tag.name);
        // ensure dir exists once
        fs::create_dir_all(Path::new(&root).join(&tag_dir)).map_err(|e| AppError::from(e).report())?;

        // Dedupe: if any file already exists for this media in this tag dir, keep one and remove the rest
        let existing_in_tag = db::files::list_by_media_and_dir(&tx, req.media_id, &tag_dir.to_string_lossy()).map_err(|e: AppError| e.report())?;

        if !existing_in_tag.is_empty() {
            // Keep the first one, remove extras
            let mut iter = existing_in_tag.into_iter();
            let _keep = iter.next();
            for extra in iter {
                let abs = Path::new(&root).join(&extra.rel_path);
                let _ = fs::remove_file(&abs);
                let _ = db::files::delete_by_id(&tx, extra.id);
            }
            // Nothing else to do if at least one exists
        } else {
            // Create exactly one link using a canonical filename
            let file_name_str = if let Some(f) = files.first() {
                if f.ext.is_empty() { f.file_name.clone() } else { format!("{}.{}", f.file_name, f.ext) }
            } else {
                // Fallback to hash.ext
                let base = media.content_hash.clone();
                let ext = guess_ext_from_objects(&root, &media.content_hash);
                if ext.is_empty() { base } else { format!("{}.{}", base, ext) }
            };
            let target_abs = Path::new(&root).join(&tag_dir).join(&file_name_str);
            let final_abs = next_available_name(&target_abs);
            fs::hard_link(&objects_abs, &final_abs).map_err(|e| AppError::from(e).report())?;
            let rel_str = crate::filesystem::path::path_to_str(final_abs.strip_prefix(&root).unwrap()).map_err(|e: AppError| e.report())?;
            let upserted = db::files::upsert(&tx, &rel_str, &final_abs).map_err(|e: AppError| e.report())?;
            let _ = db::files::update_media_id(&tx, upserted.file_entry.id, req.media_id, now_ms()).map_err(|e: AppError| e.report())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    // notify UI
    let _ = app.emit("media-updated", req.media_id);
    let _ = app.emit("library-changed", ());
    Ok(())
}

#[tauri::command]
pub fn untag_media(app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, req: TagMediaRequest) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?.clone();
    drop(root_lock);

    let mut conn = db_manager.get_connection(&root).map_err(|e: AppError| e.report())?;
    let tx = conn.transaction().map_err(|e: rusqlite::Error| AppError::Database(e).report())?;

    // Relation removal
    let changed = db::tags::remove_tag_from_media(&tx, req.media_id, req.tag_id).map_err(|e: AppError| e.report())?;

    if changed {
        // Find tag name
        let tag = db::tags::list_all(&tx).map_err(|e: AppError| e.report())?
            .into_iter().find(|t| t.id == req.tag_id)
            .ok_or_else(|| AppError::NotFound("tag not found".into()).report())?;

        // Find files for this media under Sorted/[Tag]
        let dir = format!("{}/{}", SORTED_DIRECTORY, tag.name);
        let files = db::files::list_by_media_and_dir(&tx, req.media_id, &dir).map_err(|e: AppError| e.report())?;
        for f in files {
            let abs = Path::new(&root).join(&f.rel_path);
            let _ = fs::remove_file(&abs);
            let _ = db::files::delete_by_id(&tx, f.id);
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    let _ = app.emit("media-updated", req.media_id);
    let _ = app.emit("library-changed", ());
    Ok(())
}

fn guess_ext_from_objects(root: &Path, content_hash: &str) -> String {
    // Try to find canonical object and extract its extension
    match objects::find_canonical_objects_file(root, content_hash) {
        Ok(p) => p.extension().and_then(|e| e.to_str()).unwrap_or("").to_string(),
        Err(_) => String::new()
    }
}

fn next_available_name(target: &Path) -> PathBuf {
    if !target.exists() { return target.to_path_buf(); }
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let stem = target.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = target.extension().and_then(|e| e.to_str()).unwrap_or("");
    let mut i = 2;
    loop {
        let candidate = if ext.is_empty() {
            parent.join(format!("{} ({})", stem, i))
        } else {
            parent.join(format!("{} ({}).{}", stem, i, ext))
        };
        if !candidate.exists() { return candidate; }
        i += 1;
    }
}

// ============ Review Command ============

#[tauri::command]
pub fn mark_media_reviewed(app: tauri::AppHandle, db_manager: State<'_, Arc<DbManager>>, library_root_state: State<'_, Arc<LibraryRootState>>, media_id: i64) -> Result<(), String> {
    let root_lock = library_root_state.0.lock().unwrap();
    let root = root_lock.as_ref().ok_or_else(|| "Library root not set".to_string())?.clone();
    drop(root_lock);

    let mut conn = db_manager.get_connection(&root).map_err(|e: AppError| e.report())?;
    let tx = conn.transaction().map_err(|e: rusqlite::Error| AppError::Database(e).report())?;

    // Fetch all files for this media
    let files = db::files::list_by_media_id(&tx, media_id).map_err(|e: AppError| e.report())?;

    // Remove Unsorted file links from disk and DB
    for f in files.iter().filter(|f| f.dir_path.starts_with(UNSORTED_DIRECTORY)) {
        let abs = Path::new(&root).join(&f.rel_path);
        let _ = std::fs::remove_file(&abs);
        let _ = db::files::delete_by_id(&tx, f.id);
    }

    // Set media reviewed_at at media level
    db::media::mark_reviewed(&tx, media_id, now_ms()).map_err(|e: AppError| e.report())?;

    // If there are no links under Sorted for this media, create one in Sorted root using a canonical name
    let sorted_links = db::files::list_by_media_in_dir_like(&tx, media_id, SORTED_DIRECTORY)
        .map_err(|e: AppError| e.report())?;
    if sorted_links.is_empty() {
        // Get media entry and canonical objects file
        let media = db::media::get_by_id(&tx, media_id)
            .map_err(|e: AppError| e.report())?
            .ok_or_else(|| AppError::NotFound("media not found".into()).report())?;

        let objects_abs = objects::find_canonical_objects_file(&root, &media.content_hash)
            .map_err(|e: AppError| e.report())?;

        // Ensure Sorted root exists
        let sorted_root = PathBuf::from(SORTED_DIRECTORY);
        fs::create_dir_all(Path::new(&root).join(&sorted_root))
            .map_err(|e| AppError::from(e).report())?;

        // Decide filename: prefer an existing filename from remaining records (if any), else use hash.ext
        let remaining_files = db::files::list_by_media_id(&tx, media_id).map_err(|e: AppError| e.report())?;
        let file_name_str = if let Some(f) = remaining_files.first() {
            if f.ext.is_empty() { f.file_name.clone() } else { format!("{}.{}", f.file_name, f.ext) }
        } else {
            let base = media.content_hash.clone();
            let ext = guess_ext_from_objects(&root, &media.content_hash);
            if ext.is_empty() { base } else { format!("{}.{}", base, ext) }
        };

        let target_abs = Path::new(&root).join(&sorted_root).join(&file_name_str);
        let final_abs = next_available_name(&target_abs);
        fs::hard_link(&objects_abs, &final_abs).map_err(|e| AppError::from(e).report())?;
        let rel_str = crate::filesystem::path::path_to_str(final_abs.strip_prefix(&root).unwrap())
            .map_err(|e: AppError| e.report())?;
        let upserted = db::files::upsert(&tx, &rel_str, &final_abs).map_err(|e: AppError| e.report())?;
        let _ = db::files::update_media_id(&tx, upserted.file_entry.id, media_id, now_ms())
            .map_err(|e: AppError| e.report())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    let _ = app.emit("media-updated", media_id);
    let _ = app.emit("library-changed", ());
    Ok(())
}
