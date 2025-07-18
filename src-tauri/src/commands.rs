use base64::Engine;
use crate::models::{MediaMetadata, MediaItemResponse, Bookmark, Tag};
use crate::state;
use crate::media;
use crate::database;

/// Initialize the database with a specified folder path
#[tauri::command]
pub async fn init_database(folder_path: String) -> Result<(), String> {
    state::init_database(&folder_path).await
}

/// Extract metadata from a video file, including generating a thumbnail
#[tauri::command]
pub async fn extract_video_metadata(path: String) -> Result<MediaMetadata, String> {
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Call the media function with the new connection
    media::extract_video_metadata(&path, conn).await
}

/// Extract metadata from an image file, including generating a thumbnail
#[tauri::command]
pub async fn extract_image_metadata(path: String, size: i32) -> Result<MediaMetadata, String> {
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Call the media function with the new connection
    media::extract_image_metadata(&path, size, conn).await
}

/// Get all media items
#[tauri::command]
pub async fn get_all_media() -> Result<Vec<MediaItemResponse>, String> {
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Get all media items from the database
    let media_items = database::get_all_media_items(&conn)
        .map_err(|e| format!("Failed to get media items: {}", e))?;

    let mut response_items = Vec::new();

    // For each media item, get its tags and bookmarks
    for media_item in media_items {
        let media_id = media_item.id.unwrap();

        // Get tags for this media item
        let tags = database::get_tags_for_media(&conn, media_id)
            .map_err(|e| format!("Failed to get tags: {}", e))?;

        // Get bookmarks for this media item
        let bookmarks = database::get_bookmarks_for_media(&conn, media_id)
            .map_err(|e| format!("Failed to get bookmarks: {}", e))?;

        // Get the thumbnail as a Base64 image string
        let thumbnail_base64 = match database::get_thumbnail_by_media_id(&conn, media_id, 32) {
            Ok(Some(thumbnail)) => {
                match (&thumbnail.data, &thumbnail.mime_type) {
                    (data, mime_type) => {
                        let base64_data = base64::engine::general_purpose::STANDARD.encode(data);
                        Some(format!("data:{};base64,{}", mime_type, base64_data))
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        // Create the response item
        let response_item = MediaItemResponse {
            id: media_id,
            path: media_item.path,
            title: media_item.title,
            media_type: media_item.media_type,
            length: media_item.length,
            thumbnail_base64,
            tags: tags.iter().map(|t| t.name.clone()).collect(),
            bookmarks,
        };


        response_items.push(response_item);
    }

    Ok(response_items)
}

/// Add a tag to a media item
#[tauri::command]
pub async fn add_tag(media_id: i64, tag_name: String) -> Result<(), String> {
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Create or get the tag
    let tag = Tag {
        id: None,
        name: tag_name,
    };

    let tag_id = database::insert_tag(&conn, &tag)
        .map_err(|e| format!("Failed to insert tag: {}", e))?;

    // Add the tag to the media item
    database::add_tag_to_media(&conn, media_id, tag_id)
        .map_err(|e| format!("Failed to add tag to media: {}", e))?;

    Ok(())
}

/// Add a bookmark to a media item
#[tauri::command]
pub async fn add_bookmark(media_id: i64, description: String, timestamp: i64) -> Result<i64, String> {
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Create the bookmark
    let bookmark = Bookmark {
        id: None,
        media_id,
        description,
        timestamp,
    };

    // Insert the bookmark
    let bookmark_id = database::insert_bookmark(&conn, &bookmark)
        .map_err(|e| format!("Failed to insert bookmark: {}", e))?;

    Ok(bookmark_id)
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
    // Get the database path
    let db_path = state::get_db_path()?;

    // Open a new connection for this operation
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Get the media item
    let media_item = database::get_media_item_by_id(&conn, media_id)
        .map_err(|e| format!("Failed to get media item: {}", e))?
        .ok_or_else(|| format!("Media item not found: {}", media_id))?;

    // Generate the thumbnail
    if media_item.media_type == "video" {
        media::generate_video_thumbnail(&media_item.path, size, media_id, conn).await
    } else {
        media::generate_image_thumbnail(&media_item.path, size, media_id, conn).await
    }
}

/// Scan a directory for media files and process them
/// This is a simplified version that delegates to the existing functions
#[tauri::command]
pub async fn scan_directory(folder_path: String) -> Result<(), String> {
    // For now, we'll just call the existing init_database function
    // In a real implementation, this would scan the directory and process media files
    init_database(folder_path).await?;

    Ok(())
}
