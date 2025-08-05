use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use rusqlite::{Connection, params};
use crate::database::media_items::{delete_media_item_by_path, get_all_media_items, get_media_items_by_hash, insert_media_item, update_media_item_metadata};
use crate::database::models::{MediaItem, Thumbnail};
use crate::database::thumbnails::insert_thumbnail;
use walkdir::{WalkDir, DirEntry};
use crate::media;
use crate::utils;

// Timestamp comparison tolerance to handle floating-point precision issues
// Different filesystems and OS operations can cause slight timestamp variations
const TIMESTAMP_TOLERANCE_SECONDS: f64 = 1.0;

#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    modified_at: f64,
    file_name: String,
}

#[derive(Debug)]
struct MediaMetadata {
    file_extension: String,
    media_type: String,
    video_length: Option<f64>,
}

struct CacheContext<'a> {
    path_to_files: HashMap<&'a PathBuf, &'a MediaItem>,
    seen_paths: HashSet<PathBuf>,
}

impl<'a> CacheContext<'a> {
    fn new(media_files: &'a [MediaItem]) -> Self {
        let mut path_to_files = HashMap::new();

        for media_file in media_files {
            path_to_files.insert(&media_file.path, media_file);
        }

        Self {
            path_to_files,
            seen_paths: HashSet::new(),
        }
    }

    fn mark_path_as_seen(&mut self, path: PathBuf) {
        self.seen_paths.insert(path);
    }

    fn get_existing_file(&self, path: &PathBuf) -> Option<&MediaItem> {
        self.path_to_files.get(path).copied()
    }

    fn get_orphaned_files<'b>(&self, media_files: &'b [MediaItem]) -> Vec<&'b MediaItem> {
        media_files
            .iter()
            .filter(|media_file| !self.seen_paths.contains(&media_file.path))
            .collect()
    }
}

pub fn build_cache(conn: &Connection, folder_path: String) -> Result<(), String> {
    println!("Starting cache build for folder: {}", folder_path);
    
    let media_files = get_all_media_items(conn).map_err(|e| e.to_string())?;
    println!("Found {} existing media items in database", media_files.len());
    
    let mut context = CacheContext::new(&media_files);
    
    let walker = WalkDir::new(folder_path).into_iter();
    let valid_entries = walker
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| is_valid_media_file(e));

    for entry in valid_entries {
        let file_info = extract_file_info(&entry)?;
        context.mark_path_as_seen(file_info.path.clone());
        
        match context.get_existing_file(&file_info.path) {
            Some(existing_file) => {
                process_existing_file(conn, existing_file, &file_info)?;
            }
            None => {
                process_new_file(conn, &mut context, &file_info)?;
            }
        }
    }

    cleanup_orphaned_entries(conn, &context, &media_files);
    println!("Cache build completed successfully");
    
    Ok(())
}

fn extract_file_info(entry: &DirEntry) -> Result<FileInfo, String> {
    let path = entry.path().to_path_buf();
    let metadata = entry.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?;
    
    // Convert file system timestamp to Unix epoch seconds (same format as database)
    let modified_time = metadata
        .modified()
        .map_err(|e| format!("Failed to get modified time: {}", e))?;
    
    let modified_at = modified_time
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to convert to Unix timestamp: {}", e))?
        .as_secs_f64();
    
    let size = metadata.len();
    let file_name = entry.file_name().to_string_lossy().to_string();

    Ok(FileInfo {
        path,
        size,
        modified_at,
        file_name,
    })
}

fn extract_media_metadata(path_str: &str) -> MediaMetadata {
    let file_extension = media::helpers::get_extension(path_str);
    let media_type = determine_media_type(path_str);
    let video_length = if media_type == "video" {
        media::video::get_video_duration(path_str)
    } else {
        None
    };

    MediaMetadata {
        file_extension: file_extension.to_string(),
        media_type,
        video_length,
    }
}

fn determine_media_type(path_str: &str) -> String {
    if media::video::is_video_file(path_str) {
        "video".to_string()
    } else if media::image::is_image_file(path_str) {
        "image".to_string()
    } else {
        "unknown".to_string()
    }
}

fn process_existing_file(
    conn: &Connection,
    existing_file: &MediaItem,
    file_info: &FileInfo,
) -> Result<(), String> {
    if file_needs_update(existing_file, file_info) {
        println!("File changed, updating: {}", file_info.path.to_string_lossy());
        update_file_metadata(conn, existing_file, file_info)?;
        generate_and_store_thumbnail(conn, existing_file.id, &file_info.path)?;
    }
    Ok(())
}

fn file_needs_update(existing_file: &MediaItem, file_info: &FileInfo) -> bool {
    let timestamp_diff = (existing_file.updated_at - file_info.modified_at).abs();
    let timestamp_changed = timestamp_diff > TIMESTAMP_TOLERANCE_SECONDS;
    let size_changed = existing_file.file_size != file_info.size;
    
    // Debug logging for troubleshooting timestamp issues
    if timestamp_diff > 0.0 && timestamp_diff <= TIMESTAMP_TOLERANCE_SECONDS {
        println!(
            "Timestamp difference within tolerance for {}: {:.6}s (tolerance: {}s)", 
            file_info.path.to_string_lossy(), 
            timestamp_diff, 
            TIMESTAMP_TOLERANCE_SECONDS
        );
    }
    
    if timestamp_changed || size_changed {
        println!(
            "File needs update - {}: timestamp_diff={:.6}s (db: {:.6}, file: {:.6}, changed: {}), size_diff={} (changed: {})",
            file_info.path.to_string_lossy(),
            timestamp_diff,
            existing_file.updated_at,
            file_info.modified_at,
            timestamp_changed,
            (file_info.size as i64) - (existing_file.file_size as i64),
            size_changed
        );
    }
    
    timestamp_changed || size_changed
}

fn file_needs_update_for_renamed_file(existing_file: &MediaItem, file_info: &FileInfo) -> bool {
    let timestamp_diff = (existing_file.updated_at - file_info.modified_at).abs();
    let timestamp_changed = timestamp_diff > TIMESTAMP_TOLERANCE_SECONDS;
    let size_changed = existing_file.file_size != file_info.size;
    
    if timestamp_changed || size_changed {
        println!(
            "Renamed file needs update - {}: timestamp_diff={:.6}s, size_diff={}",
            file_info.path.to_string_lossy(),
            timestamp_diff,
            (file_info.size as i64) - (existing_file.file_size as i64)
        );
    }
    
    timestamp_changed || size_changed
}

fn update_media_item_path_direct(conn: &Connection, old_path: &str, new_path: &str) -> Result<(), rusqlite::Error> {
    // Get the filename from the new path to update the title
    let new_title = Path::new(new_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Update the media item
    let now = utils::get_current_timestamp();
    conn.execute(
        "UPDATE media_items SET path = ?1, file_name = ?2, updated_at = ?3 WHERE path = ?4",
        params![new_path, new_title, now, old_path],
    )?;

    Ok(())
}

fn update_file_metadata(
    conn: &Connection,
    _existing_file: &MediaItem,
    file_info: &FileInfo,
) -> Result<(), String> {
    let hash = media::hash::hash_path(&file_info.path)?;
    update_media_item_metadata(
        conn,
        &file_info.path.to_string_lossy(),
        &hash.to_string(),
        file_info.size,
        file_info.modified_at,
    )
    .map_err(|e| e.to_string())
    .map(|_| ())
}

fn process_new_file(
    conn: &Connection,
    context: &mut CacheContext,
    file_info: &FileInfo,
) -> Result<(), String> {
    let hash = media::hash::hash_path(&file_info.path)?;
    let path_str = file_info.path.to_string_lossy();

    // Check if files with this hash already exist (handles duplicates)
    let existing_files = get_media_items_by_hash(conn, &hash.to_string()).map_err(|e| e.to_string())?;
    
    if !existing_files.is_empty() {
        // Find which file with this hash no longer exists (indicating a rename)
        let mut renamed_file = None;
        for existing_file in &existing_files {
            if !std::fs::exists(&existing_file.path).unwrap_or(true) {
                renamed_file = Some(existing_file);
                break;
            }
        }
        
        match renamed_file {
            Some(existing_file) => {
                // File was renamed - update the existing record
                println!("File renamed detected - updating path from {} to {}", 
                    existing_file.path.to_string_lossy(), path_str);
                
                // Update the path in the database
                update_media_item_path_direct(conn, &existing_file.path.to_string_lossy(), &path_str)
                    .map_err(|e| e.to_string())?;
                
                // Mark the old path as seen to prevent it from being marked as orphaned
                context.mark_path_as_seen(existing_file.path.clone());
                
                // Update metadata if file changed
                if file_needs_update_for_renamed_file(&existing_file, file_info) {
                    update_file_metadata(conn, &existing_file, file_info)?;
                    generate_and_store_thumbnail(conn, existing_file.id, &file_info.path)?;
                }
            }
            None => {
                // All files with this hash still exist, so this is a new duplicate
                let metadata = extract_media_metadata(&path_str);
                let media_item = create_media_item(file_info, metadata, hash.to_string());
                
                println!("Inserting new duplicate file: {}", path_str);
                let media_id = insert_media_item(conn, &media_item).map_err(|e| e.to_string())?;
                
                generate_and_store_thumbnail(conn, media_id, &file_info.path)?;
            }
        }
    } else {
        // Truly new file (no existing files with this hash)
        let metadata = extract_media_metadata(&path_str);
        let media_item = create_media_item(file_info, metadata, hash.to_string());
        
        println!("Inserting new media item: {}", path_str);
        let media_id = insert_media_item(conn, &media_item).map_err(|e| e.to_string())?;
        
        generate_and_store_thumbnail(conn, media_id, &file_info.path)?;
    }
    
    Ok(())
}

fn create_media_item(file_info: &FileInfo, metadata: MediaMetadata, hash: String) -> MediaItem {
    let now = utils::get_current_timestamp();
    
    MediaItem {
        id: 0,
        path: file_info.path.clone(),
        file_name: file_info.file_name.clone(),
        file_size: file_info.size,
        file_extension: metadata.file_extension,
        media_type: metadata.media_type,
        video_length: metadata.video_length,
        hash,
        created_at: now,
        updated_at: file_info.modified_at,
    }
}

fn generate_and_store_thumbnail(
    conn: &Connection,
    media_id: i64,
    path: &PathBuf,
) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    
    let thumbnail_result = generate_thumbnail(&path_str)?;
    let thumbnail = Thumbnail {
        media_id,
        data: thumbnail_result,
        mime_type: "image/webp".to_string(),
    };
    
    match insert_thumbnail(conn, &thumbnail) {
        Ok(_) => {
            println!("Thumbnail generated and stored for: {}", path_str);
            Ok(())
        }
        Err(e) => {
            println!("Failed to store thumbnail for {}: {}", path_str, e);
            Err(format!("Thumbnail storage failed: {}", e))
        }
    }
}

fn generate_thumbnail(path_str: &str) -> Result<Vec<u8>, String> {
    if media::video::is_video_file(path_str) {
        media::video::generate_video_thumbnail(path_str)
    } else if media::image::is_image_file(path_str) {
        media::image::generate_image_thumbnail(path_str)
    } else {
        Err("Unsupported media type for thumbnail generation".to_string())
    }
}

fn cleanup_orphaned_entries(conn: &Connection, context: &CacheContext, media_files: &[MediaItem]) {
    let orphaned_files = context.get_orphaned_files(media_files);
    
    if orphaned_files.is_empty() {
        println!("No orphaned entries found");
        return;
    }
    
    println!("Found {} orphaned entries:", orphaned_files.len());
    for media_file in orphaned_files {
        println!("  - {}", media_file.path.to_string_lossy());
        delete_media_item_by_path(conn, &media_file.path.to_string_lossy()).map_err(|e| e.to_string()).unwrap_or_else(|e|
            println!("Failed to delete orphaned entry: {}", e)
        )
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_valid_media_file(entry: &DirEntry) -> bool {
    let Ok(metadata) = entry.metadata() else {
        return false;
    };
    
    if !metadata.is_file() {
        return false;
    }

    let path = entry.path().to_string_lossy();
    media::image::is_image_file(&path) || media::video::is_video_file(&path)
}