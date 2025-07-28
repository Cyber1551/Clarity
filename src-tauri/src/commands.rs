use base64::Engine;
use crate::models::{MediaMetadata, UpdateStats, MediaItem};
use crate::types::{MediaItemResponse};
use crate::state;
use crate::media;
use crate::database;
use std::path::Path;
use std::collections::HashMap;
use std::fs;

/// Initialize the database with a specified folder path
#[tauri::command]
pub async fn initialize_database(folder_path: String) -> Result<(), String> {
    state::initialize_database(&folder_path).await
}

/// Get all media items
#[tauri::command]
pub async fn get_all_media() -> Result<Vec<MediaItemResponse>, String> {
    // Get all media items from the database using spawn_blocking
    let media_items = tauri::async_runtime::spawn_blocking(move || {
        // Get a connection from the pool for this operation
        let conn = state::get_connection()?;
        database::get_all_media_items(&conn)
            .map_err(|e| format!("Failed to get media items: {}", e))
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

    let mut response_items = Vec::new();

    // For each media item, get its tags and bookmarks
    for media_item in media_items {
        let media_id = media_item.id.unwrap();

        let thumbnail_result = tauri::async_runtime::spawn_blocking(move || {
            database::get_thumbnail_by_media_id(&conn, &media_id)
                .map_err(|e| format!("Failed to get thumbnail: {}", e))
        }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

        let thumbnail_base64 = match thumbnail_result {
            Some(thumbnail) => {
                // Thumbnail exists, encode it as base64
                let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);
                Some(format!("data:{};base64,{}", thumbnail.mime_type, base64_data))
            }
            None => {
                // Thumbnail doesn't exist, generate it
                // Open a new connection for this operation
                let media_item_conn = rusqlite::Connection::open(&db_path)
                    .map_err(|e| format!("Failed to open database for media item: {}", e))?;

                let media_id_clone = media_id;
                let media_item_result = tauri::async_runtime::spawn_blocking(move || {
                    database::get_media_item_by_id(&media_item_conn, media_id_clone)
                        .map_err(|e| format!("Failed to get media item: {}", e))
                        .and_then(|opt| opt.ok_or_else(|| format!("Media item not found: {}", media_id_clone)))
                }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

                // Generate the thumbnail based on media type
                // Open a new connection for thumbnail generation
                let thumbnail_gen_conn = rusqlite::Connection::open(&db_path)
                    .map_err(|e| format!("Failed to open database for thumbnail generation: {}", e))?;

                let thumbnail_data_url = if media_item_result.media_type == "video" {
                    media::generate_video_thumbnail(&media_item_result.path, 256, media_id, thumbnail_gen_conn)
                        .await
                        .map_err(|e| format!("Failed to generate video thumbnail: {}", e))?
                } else {
                    // Open another new connection for image thumbnail generation
                    let img_thumbnail_conn = rusqlite::Connection::open(&db_path)
                        .map_err(|e| format!("Failed to open database for image thumbnail generation: {}", e))?;

                    media::generate_image_thumbnail(&media_item_result.path, 256, media_id, img_thumbnail_conn)
                        .await
                        .map_err(|e| format!("Failed to generate image thumbnail: {}", e))?
                };

                Some(thumbnail_data_url)
            }
        };

        // Create the response item
        let response_item = MediaItemResponse {
            id: media_id,
            path: media_item.path,
            title: media_item.title,
            media_type: media_item.media_type,
            length: media_item.video_length,
            thumbnail_base64,
            tags: tags.iter().map(|t| t.name.clone()).collect(),
            bookmarks,
        };

        response_items.push(response_item);
    }

    Ok(response_items)
}


pub async fn get_media_items() -> Result<Vec<MediaItemResponse>, String> {
    let response_items = tauri::async_runtime::spawn_blocking(move || {
        let conn = state::get_connection()?;
        let media_items = database::get_all_media_items(&conn).map_err(|e| format!("Failed to get media items: {}", e))?;

        let mut responses = Vec::new();

        for media in media_items {
            //let tags = database::get_tags(&conn, media.id)?;
            //let bookmarks = database::get_bookmarks(&conn, media.id)?;
            let thumbnail = database::get_thumbnail_base64(&conn, media.id)?;

            responses.push(MediaItemResponse {
                id: media.id,
                path: media.path,
                file_name: media.file_name,
                file_size: media.file_size,
                file_extension: media.file_extension,
                media_type: media.media_type,
                video_length: media.video_length,
                thumbnail_base64: thumbnail
            });
        }

        Ok::<_, String>(responses)
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
    
    Ok(response_items)
}




/// Extract metadata from a video file, including generating a thumbnail
#[tauri::command]
pub async fn extract_video_metadata(path: String) -> Result<MediaMetadata, String> {
    // Get a database connection
    let conn = state::get_db_connection()?;

    // Call the media function with the connection
    // Pass the connection directly (not as a reference) to make it Send
    media::extract_video_metadata(&path, conn).await
}

/// Extract metadata from an image file, including generating a thumbnail
#[tauri::command]
pub async fn extract_image_metadata(path: String, size: i32) -> Result<MediaMetadata, String> {
    // Note: size parameter is kept for backward compatibility but a fixed size is used internally
    // Get a database connection
    let conn = state::get_db_connection()?;

    // Call the media function with the connection
    // Pass the connection directly (not as a reference) to make it Send
    media::extract_image_metadata(&path, size, conn).await
}

/// Delete a media item and all its associated data by path
#[tauri::command]
pub async fn delete_media_item_by_path(path: String) -> Result<bool, String> {
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Delete the media item and its associated data
    let result = database::delete_media_item_by_path(&mut conn, &path)
        .map_err(|e| format!("Failed to delete media item: {}", e))?;

    Ok(result)
}

/// Update the path of a media item (for handling renamed files)
#[tauri::command]
pub async fn update_media_item_path(old_path: String, new_path: String) -> Result<bool, String> {
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let mut conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Update the media item path
    let result = database::update_media_item_path(&mut conn, &old_path, &new_path)
        .map_err(|e| format!("Failed to update media item path: {}", e))?;

    Ok(result)
}

/// Check if a thumbnail exists at a specific size
#[tauri::command]
pub async fn check_thumbnail_exists(media_id: i64, size: i32) -> Result<bool, String> {
    // Note: size parameter is kept for backward compatibility but a fixed size is used internally
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Check if the thumbnail exists
    let thumbnail = database::get_thumbnail_by_media_id(&conn, media_id, size)
        .map_err(|e| format!("Failed to check thumbnail: {}", e))?;

    Ok(thumbnail.is_some())
}

/// Get a thumbnail at a specific size
#[tauri::command]
pub async fn get_thumbnail(media_id: i64, size: i32) -> Result<String, String> {
    // Note: size parameter is kept for backward compatibility but a fixed size is used internally
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Get the thumbnail
    let thumbnail = database::get_thumbnail_by_media_id(&conn, media_id, size)
        .map_err(|e| format!("Failed to get thumbnail: {}", e))?
        .ok_or_else(|| format!("Thumbnail not found for media_id: {}", media_id))?;

    // Convert to base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);

    // Create a data URL
    let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);

    Ok(data_url)
}

/// Generate a thumbnail at a specific size
#[tauri::command]
pub async fn generate_thumbnail(media_id: i64, size: i32) -> Result<String, String> {
    // Note: size parameter is kept for backward compatibility but a fixed size is used internally
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for getting the media item
    let media_item_conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database for media item: {}", e))?;

    // Get the media item using spawn_blocking
    let media_id_clone = media_id;
    let media_item = tauri::async_runtime::spawn_blocking(move || {
        database::get_media_item_by_id(&media_item_conn, media_id_clone)
            .map_err(|e| format!("Failed to get media item: {}", e))
            .and_then(|opt| opt.ok_or_else(|| format!("Media item not found: {}", media_id_clone)))
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

    // Generate the thumbnail
    // Open a new connection for thumbnail generation
    if media_item.media_type == "video" {
        // Open a new connection for video thumbnail generation
        let video_thumbnail_conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database for video thumbnail generation: {}", e))?;
            
        media::generate_video_thumbnail(&media_item.path, size, media_id, video_thumbnail_conn).await
    } else {
        // Open a new connection for image thumbnail generation
        let image_thumbnail_conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database for image thumbnail generation: {}", e))?;
            
        media::generate_image_thumbnail(&media_item.path, size, media_id, image_thumbnail_conn).await
    }
}

/// Scan a directory for media files
async fn scan_directory_internal(directory: &str) -> Result<Vec<MediaItem>, String> {
    let mut media_items = Vec::new();
    
    let entries = fs::read_dir(directory)
        .map_err(|e| format!("Failed to read directory {}: {}", directory, e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        
        if path.is_dir() {
            if !should_skip_directory(&path_str) {
                let future = Box::pin(scan_directory_internal(&path_str));
                let sub_items = future.await?;
                media_items.extend(sub_items);
            }
        } else if path.is_file() {
            if let Some(media_item) = create_media_item(&path_str) {
                media_items.push(media_item);
            }
        }
    }
    
    Ok(media_items)
}

/// Create maps of media items for quick lookup
fn create_media_item_maps(items: &[MediaItem]) -> (HashMap<String, &MediaItem>, HashMap<String, &MediaItem>) {
    let mut by_path = HashMap::new();
    let mut by_filename = HashMap::new();
    
    for item in items {
        by_path.insert(item.path.clone(), item);
        
        if let Some(filename) = Path::new(&item.path).file_name() {
            if let Some(filename_str) = filename.to_str() {
                by_filename.insert(filename_str.to_string(), item);
            }
        }
    }
    
    (by_path, by_filename)
}

/// Handle deleted and renamed files
async fn handle_deleted_and_renamed_files(
    _conn: &mut rusqlite::Connection,
    existing_items: &[MediaItem],
    existing_by_path: &HashMap<String, &MediaItem>,
    scanned_by_filename: &HashMap<String, &MediaItem>
) -> Result<(i32, i32), String> {
    let mut deleted_count = 0;
    let mut renamed_count = 0;
    
    for existing_item in existing_items {
        let path = &existing_item.path;
        let path_exists = Path::new(path).exists();
        
        if !path_exists {
            // Check if this might be a renamed file
            let filename = Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_string();
            
            if let Some(scanned_item) = scanned_by_filename.get(&filename) {
                if !existing_by_path.contains_key(&scanned_item.path) {
                    // This is likely a renamed file (same filename, different path)
                    // Use spawn_blocking to update the media item path
                    let path_clone = path.clone();
                    let scanned_path_clone = scanned_item.path.clone();
                    
                    // Get a connection from the pool for this operation
                    let mut update_conn = state::get_db_connection()?;
                    
                    tauri::async_runtime::spawn_blocking(move || {
                        database::update_media_item_path(&mut update_conn, &path_clone, &scanned_path_clone)
                            .map_err(|e| format!("Failed to update media item path: {}", e))
                    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
                    
                    renamed_count += 1;
                }
            } else {
                // File was deleted
                // Use spawn_blocking to delete the media item
                let path_clone = path.clone();
                
                // Get a connection from the pool for this operation
                let mut delete_conn = state::get_db_connection()?;
                
                tauri::async_runtime::spawn_blocking(move || {
                    database::delete_media_item_by_path(&mut delete_conn, &path_clone)
                        .map_err(|e| format!("Failed to delete media item: {}", e))
                }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
                
                deleted_count += 1;
            }
        }
    }
    
    Ok((deleted_count, renamed_count))
}

/// Process media files to extract metadata
async fn process_media_files(
    scanned_items: &[MediaItem]
) -> Result<(i32, i32), String> {
    let mut processed_video_count = 0;
    let mut processed_image_count = 0;
    
    for item in scanned_items {
        // Check if the item already exists in the database using spawn_blocking
        // Get a connection from the pool for this operation
        let check_conn = state::get_connection()?;
            
        let path_clone = item.path.clone();
        let existing_item = tauri::async_runtime::spawn_blocking(move || {
            database::get_media_item_by_path(&check_conn, &path_clone)
                .map_err(|e| format!("Failed to check if media item exists: {}", e))
        }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
        
        if existing_item.is_none() {
            if item.media_type == "video" {
                // Get a connection from the pool for video metadata extraction
                let video_conn = state::get_db_connection()?;
                
                // Extract video metadata
                media::extract_video_metadata(&item.path, video_conn).await?;
                processed_video_count += 1;
            } else if item.media_type == "image" {
                // Get a connection from the pool for image metadata extraction
                let image_conn = state::get_db_connection()?;
                
                // Extract image metadata
                media::extract_image_metadata(&item.path, 256, image_conn).await?;
                processed_image_count += 1;
            }
        }
    }
    
    Ok((processed_video_count, processed_image_count))
}

/// Scan a directory for media files and return the scanned items
#[tauri::command]
pub async fn scan_directory(folder_path: String) -> Result<Vec<MediaItem>, String> {
    scan_directory_internal(&folder_path).await
}

/// Update the media cache by scanning a directory and updating the database
#[tauri::command]
pub async fn update_media_cache(folder_path: String) -> Result<UpdateStats, String> {
    // Get a connection from the pool for getting all existing media items
    let media_items_conn = state::get_connection()?;
    
    // 1. Get all existing media items from the database using spawn_blocking
    let existing_items = tauri::async_runtime::spawn_blocking(move || {
        database::get_all_media_items(&media_items_conn)
            .map_err(|e| format!("Failed to get media items: {}", e))
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
    
    // 2. Create maps for quick lookup
    let (existing_by_path, _) = create_media_item_maps(&existing_items);
    
    // 3. Scan the directory for media files
    let scanned_items = scan_directory_internal(&folder_path).await?;
    
    // 4. Create maps for quick lookup of scanned items
    let (_, scanned_by_filename) = create_media_item_maps(&scanned_items);
    
    // 5. Handle deleted and renamed files
    // Get a mutable connection from the pool for updates
    let mut conn_for_updates = state::get_db_connection()?;
    
    let (deleted_count, renamed_count) = handle_deleted_and_renamed_files(
        &mut conn_for_updates, 
        &existing_items, 
        &existing_by_path, 
        &scanned_by_filename
    ).await?;
    
    // 6. Process media files to extract metadata
    let (processed_video_count, processed_image_count) = process_media_files(&scanned_items).await?;
    
    // 7. Return statistics about what was updated
    Ok(UpdateStats {
        scanned_count: scanned_items.len() as i32,
        deleted_count,
        renamed_count,
        processed_video_count,
        processed_image_count,
    })
}
