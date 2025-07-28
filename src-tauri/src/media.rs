use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::api::process::Command;
use base64::Engine;
use std::time::SystemTime;

use crate::models::{MediaMetadata, MediaItem, Thumbnail};
use crate::database;
use crate::state;
use rusqlite::Connection;

/// Extract metadata from a video file, including generating a thumbnail
pub async fn extract_video_metadata(
    path: &str,
    conn: Connection
) -> Result<MediaMetadata, String> {
    // Extract the filename from the path to use as the title
    let video_path = PathBuf::from(path);
    let title = video_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Get file metadata to check modification time
    let file_metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    
    // Get the file's modification time as a Unix timestamp
    let file_modified_time = file_metadata.modified()
        .map_err(|e| format!("Failed to get file modification time: {}", e))?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Failed to calculate file modification time: {}", e))?
        .as_secs() as i64;

    // Use spawn_blocking to perform database operations
    let path_clone = path.to_string();
    let existing_media = tauri::async_runtime::spawn_blocking(move || {
        database::get_media_item_by_path(&conn, &path_clone)
            .map_err(|e| format!("Database error: {}", e))
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

    let size = 256; // Standard thumbnail size
    let mut thumbnail_data: Vec<u8> = Vec::new();
    let mut duration_sec: u32 = 0;
    let mut needs_thumbnail_update = false;
    let media_id: i64;

    // Get the database path for creating new connections
    let db_path = state::get_db_path()?;

    // Process based on whether the media item exists and if it's been modified
    if let Some(media) = existing_media {
        media_id = media.id.unwrap();
        
        // Media item exists, check if it needs to be updated
        if file_modified_time > media.updated_at {
            // File has been modified since last update, need to regenerate thumbnail
            needs_thumbnail_update = true;
            
            // Generate a temporary thumbnail file name
            let temp_thumbnail_path = format!("{}.thumb.jpg", path);

            // Run ffmpeg to extract a frame
            let ffmpeg_cmd = Command::new_sidecar("ffmpeg")
                .map_err(|e| format!("Failed to create ffmpeg sidecar command: {}", e))?;
            let ffmpeg_status = ffmpeg_cmd
                .args(&[
                    "-y", "-ss", "00:00:01",
                    "-i", path,
                    "-frames:v", "1",
                    "-q:v", "8", "-vf", "scale=320:-1",
                    &temp_thumbnail_path,
                ])
                .status()
                .map_err(|e| format!("Failed to run ffmpeg sidecar: {}", e))?;
            if !ffmpeg_status.success() {
                return Err(format!("ffmpeg sidecar failed with status: {:?}", ffmpeg_status));
            }

            // Run ffprobe to get duration
            let ffprobe_cmd = Command::new_sidecar("ffprobe")
                .map_err(|e| format!("Failed to create ffprobe sidecar command: {}", e))?;
            let ffprobe_output = ffprobe_cmd
                .args(&[
                    "-v", "error",
                    "-select_streams", "v:0",
                    "-show_entries", "format=duration",
                    "-of", "default=noprint_wrappers=1:nokey=1",
                    path,
                ])
                .output()
                .map_err(|e| format!("Failed to run ffprobe sidecar: {}", e))?;
            if !ffprobe_output.status.success() {
                return Err(format!("ffprobe sidecar failed with status: {:?}", ffprobe_output.status));
            }

            // Parse the duration
            let duration_str = String::from_utf8_lossy(ffprobe_output.stdout.as_bytes()).trim().to_string();
            let duration: f64 = duration_str
                .parse()
                .map_err(|e| format!("Failed to parse duration: {}", e))?;
            duration_sec = duration.round() as u32;

            // Read the thumbnail file into memory
            thumbnail_data = fs::read(&temp_thumbnail_path)
                .map_err(|e| format!("Failed to read thumbnail file: {}", e))?;

            // Clean up the temporary thumbnail file
            let _ = fs::remove_file(&temp_thumbnail_path); // Ignore errors here

            // Update the media item with new timestamp and duration
            let now = database::get_current_timestamp();
            let media_id_clone = media_id;
            let duration_sec_clone = duration_sec;
            
            // Open a new connection for this operation
            let update_conn = rusqlite::Connection::open(&db_path)
                .map_err(|e| format!("Failed to open database for update: {}", e))?;
                
            tauri::async_runtime::spawn_blocking(move || {
                update_conn.execute(
                    "UPDATE media_items SET updated_at = ?1, length = ?2 WHERE id = ?3",
                    rusqlite::params![now, duration_sec_clone as i64, media_id_clone],
                ).map_err(|e| format!("Failed to update media item timestamp: {}", e))
            }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
        } else {
            // File hasn't been modified, no need to update thumbnail
            println!("Video file hasn't been modified, using existing thumbnail for {}", path);
            duration_sec = media.video_length.unwrap_or(0) as u32;
        }
    } else {
        // Media item doesn't exist, need to create it and generate thumbnail
        needs_thumbnail_update = true;
        
        // Generate a temporary thumbnail file name
        let temp_thumbnail_path = format!("{}.thumb.jpg", path);

        // Run ffmpeg to extract a frame
        let ffmpeg_cmd = Command::new_sidecar("ffmpeg")
            .map_err(|e| format!("Failed to create ffmpeg sidecar command: {}", e))?;
        let ffmpeg_status = ffmpeg_cmd
            .args(&[
                "-y", "-ss", "00:00:01",
                "-i", path,
                "-frames:v", "1",
                "-q:v", "8", "-vf", "scale=320:-1",
                &temp_thumbnail_path,
            ])
            .status()
            .map_err(|e| format!("Failed to run ffmpeg sidecar: {}", e))?;
        if !ffmpeg_status.success() {
            return Err(format!("ffmpeg sidecar failed with status: {:?}", ffmpeg_status));
        }

        // Run ffprobe to get duration
        let ffprobe_cmd = Command::new_sidecar("ffprobe")
            .map_err(|e| format!("Failed to create ffprobe sidecar command: {}", e))?;
        let ffprobe_output = ffprobe_cmd
            .args(&[
                "-v", "error",
                "-select_streams", "v:0",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
                path,
            ])
            .output()
            .map_err(|e| format!("Failed to run ffprobe sidecar: {}", e))?;
        if !ffprobe_output.status.success() {
            return Err(format!("ffprobe sidecar failed with status: {:?}", ffprobe_output.status));
        }

        // Parse the duration
        let duration_str = String::from_utf8_lossy(ffprobe_output.stdout.as_bytes()).trim().to_string();
        let duration: f64 = duration_str
            .parse()
            .map_err(|e| format!("Failed to parse duration: {}", e))?;
        duration_sec = duration.round() as u32;

        // Read the thumbnail file into memory
        thumbnail_data = fs::read(&temp_thumbnail_path)
            .map_err(|e| format!("Failed to read thumbnail file: {}", e))?;

        // Clean up the temporary thumbnail file
        let _ = fs::remove_file(&temp_thumbnail_path); // Ignore errors here

        // Create new media item
        let media_item = MediaItem {
            id: None,
            path: path.to_string(),
            title: title.clone(),
            media_type: "video".to_string(),
            video_length: Some(duration_sec as i64),
            created_at: database::get_current_timestamp(),
            updated_at: file_modified_time, // Use the file's modification time
        };

        // Insert the media item in a blocking task
        // Open a new connection for this operation
        let insert_conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database for insert: {}", e))?;
            
        let media_item_clone = media_item.clone();
        media_id = tauri::async_runtime::spawn_blocking(move || {
            database::insert_media_item(&insert_conn, &media_item_clone)
                .map_err(|e| format!("Failed to insert media item: {}", e))
        }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
    };

    // Only update the thumbnail if needed
    if needs_thumbnail_update {
        let thumbnail = Thumbnail {
            media_id,
            size,
            data: thumbnail_data.clone(),
            mime_type: "image/jpeg".to_string(),
        };

        // Insert the thumbnail in a blocking task
        // Open a new connection for this operation
        let thumbnail_conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database for thumbnail insert: {}", e))?;
            
        let thumbnail_clone = thumbnail.clone();
        tauri::async_runtime::spawn_blocking(move || {
            database::insert_thumbnail(&thumbnail_conn, &thumbnail_clone)
                .map_err(|e| format!("Failed to insert thumbnail: {}", e))
        }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
    }

    // Get the thumbnail data to encode as base64
    // Open a new connection for this operation
    let get_thumbnail_conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database for thumbnail retrieval: {}", e))?;
        
    let media_id_clone = media_id;
    let size_clone = size;
    let thumbnail = tauri::async_runtime::spawn_blocking(move || {
        database::get_thumbnail_by_media_id(&get_thumbnail_conn, media_id_clone, size_clone)
            .map_err(|e| format!("Failed to get thumbnail: {}", e))
            .and_then(|opt| opt.ok_or_else(|| format!("Thumbnail not found for media_id: {}", media_id_clone)))
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

    // Encode the thumbnail data as base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);
    let thumbnail_base64 = format!("data:image/jpeg;base64,{}", base64_data);

    // Return the metadata with the database IDs
    Ok(MediaMetadata {
        id: media_id,
        duration: duration_sec as i32,
        thumbnail_base64,
        thumbnail_size: size
    })
}

/// Extract metadata from an image file, including generating a thumbnail
pub async fn extract_image_metadata(
     path: &str,
     _size: i32, // Parameter kept for backward compatibility but not used
     conn: Connection
 ) -> Result<MediaMetadata, String> {
     // Use standard thumbnail size
     let size = 256;
     // Extract the filename from the path to use as the title
     let image_path = PathBuf::from(path);
     let title = image_path
         .file_name()
         .and_then(|name| name.to_str())
         .unwrap_or("Unknown")
         .to_string();

     // Get file metadata to check modification time
     let file_metadata = fs::metadata(path)
         .map_err(|e| format!("Failed to get file metadata: {}", e))?;
     
     // Get the file's modification time as a Unix timestamp
     let file_modified_time = file_metadata.modified()
         .map_err(|e| format!("Failed to get file modification time: {}", e))?
         .duration_since(SystemTime::UNIX_EPOCH)
         .map_err(|e| format!("Failed to calculate file modification time: {}", e))?
         .as_secs() as i64;

     // Use spawn_blocking to perform database operations
     let path_clone = path.to_string();
     let existing_media = tauri::async_runtime::spawn_blocking(move || {
         database::get_media_item_by_path(&conn, &path_clone)
             .map_err(|e| format!("Database error: {}", e))
     }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

     // Get the database path for creating new connections
     let db_path = state::get_db_path()?;

     // Determine the MIME type based on file extension
     let mime_type = if path.to_lowercase().ends_with(".jpg") || path.to_lowercase().ends_with(".jpeg") {
         "image/jpeg"
     } else if path.to_lowercase().ends_with(".png") {
         "image/png"
     } else if path.to_lowercase().ends_with(".gif") {
         "image/gif"
     } else {
         "application/octet-stream" // Default MIME type
     };

     let mut thumbnail_data: Vec<u8> = Vec::new();
     let mut needs_thumbnail_update = false;
     let mut media_id: i64;

     // Process based on whether the media item exists and if it's been modified
     if let Some(media) = existing_media {
         media_id = media.id.unwrap();
         
         // Media item exists, check if it needs to be updated
         if file_modified_time > media.updated_at {
             // File has been modified since last update, need to regenerate thumbnail
             needs_thumbnail_update = true;
             
             // Read the image file into memory
             thumbnail_data = fs::read(path)
                 .map_err(|e| format!("Failed to read image file: {}", e))?;
             
             // Update the media item with new timestamp
             let now = database::get_current_timestamp();
             let media_id_clone = media_id;
             
             // Open a new connection for this operation
             let update_conn = rusqlite::Connection::open(&db_path)
                 .map_err(|e| format!("Failed to open database for update: {}", e))?;
                 
             tauri::async_runtime::spawn_blocking(move || {
                 update_conn.execute(
                     "UPDATE media_items SET updated_at = ?1 WHERE id = ?2",
                     rusqlite::params![now, media_id_clone],
                 ).map_err(|e| format!("Failed to update media item timestamp: {}", e))
             }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
         } else {
             // File hasn't been modified, no need to update thumbnail
             println!("Image file hasn't been modified, using existing thumbnail for {}", path);
         }
     } else {
         // Media item doesn't exist, need to create it and generate thumbnail
         needs_thumbnail_update = true;
         
         // Read the image file into memory
         thumbnail_data = fs::read(path)
             .map_err(|e| format!("Failed to read image file: {}", e))?;
         
         // Create new media item
         let media_item = MediaItem {
             id: None,
             path: path.to_string(),
             file_name: title.clone(),
             file_size: 32,
             file_extension: String::from("image"),
             video_length: None, // Images don't have a duration
             created_at: database::get_current_timestamp(),
             updated_at: file_modified_time, // Use the file's modification time
         };

         // Insert the media item in a blocking task
         // Open a new connection for this operation
         let insert_conn = rusqlite::Connection::open(&db_path)
             .map_err(|e| format!("Failed to open database for insert: {}", e))?;
             
         let media_item_clone = media_item.clone();
         media_id = tauri::async_runtime::spawn_blocking(move || {
             database::insert_media_item(&insert_conn, &media_item_clone)
                 .map_err(|e| format!("Failed to insert media item: {}", e))
         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
     };

     // Only update the thumbnail if needed
     if needs_thumbnail_update {
         let thumbnail = Thumbnail {
             media_id,
             size,
             data: thumbnail_data.clone(),
             mime_type: mime_type.to_string(),
         };

         println!("Inserting/Updating thumbnail for media_id {:?}", media_id);

         // Insert the thumbnail in a blocking task
         // Open a new connection for this operation
         let thumbnail_conn = rusqlite::Connection::open(&db_path)
             .map_err(|e| format!("Failed to open database for thumbnail insert: {}", e))?;
             
         let thumbnail_clone = thumbnail.clone();
         tauri::async_runtime::spawn_blocking(move || {
             database::insert_thumbnail(&thumbnail_conn, &thumbnail_clone)
                 .map_err(|e| format!("Failed to insert thumbnail: {}", e))
         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
     }

     // Get the thumbnail data to encode as base64
     // Open a new connection for this operation
     let get_thumbnail_conn = rusqlite::Connection::open(&db_path)
         .map_err(|e| format!("Failed to open database for thumbnail retrieval: {}", e))?;
         
     let media_id_clone = media_id;
     let size_clone = size;
     let thumbnail = tauri::async_runtime::spawn_blocking(move || {
         database::get_thumbnail_by_media_id(&get_thumbnail_conn, media_id_clone, size_clone)
             .map_err(|e| format!("Failed to get thumbnail: {}", e))
             .and_then(|opt| opt.ok_or_else(|| format!("Thumbnail not found for media_id: {}", media_id_clone)))
     }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

     // Encode the thumbnail data as base64
     let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);
     let thumbnail_base64 = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);

     // Return the metadata
     Ok(MediaMetadata {
         id: media_id,
         thumbnail_base64,
         thumbnail_size: size,
         duration: 0, // Images don't have a duration
     })
 }

/// Generate a thumbnail for a video at a specific size
// pub async fn generate_video_thumbnail(
//     path: &str,
//     _size: i32, // Parameter kept for backward compatibility but not used
//     media_id: i64,
//     conn: Connection
// ) -> Result<String, String> {
//     // Use standard thumbnail size
//     let size = 256;
//
//     // Get file metadata to check modification time
//     let file_metadata = fs::metadata(path)
//         .map_err(|e| format!("Failed to get file metadata: {}", e))?;
//
//     // Get the file's modification time as a Unix timestamp
//     let file_modified_time = file_metadata.modified()
//         .map_err(|e| format!("Failed to get file modification time: {}", e))?
//         .duration_since(SystemTime::UNIX_EPOCH)
//         .map_err(|e| format!("Failed to calculate file modification time: {}", e))?
//         .as_secs() as i64;
//
//     // Get the database path for creating new connections
//     let db_path = state::get_db_path()?;
//
//     // Check if we need to regenerate the thumbnail using spawn_blocking
//     let media_id_clone = media_id;
//     let size_clone = size;
//     let existing_thumbnail = tauri::async_runtime::spawn_blocking(move || {
//         database::get_thumbnail_by_media_id(&conn, media_id_clone, size_clone)
//             .map_err(|e| format!("Database error: {}", e))
//     }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//     // Get the media item to check its updated_at timestamp using spawn_blocking
//     // Open a new connection for this operation
//     let get_media_conn = rusqlite::Connection::open(&db_path)
//         .map_err(|e| format!("Failed to open database for media item retrieval: {}", e))?;
//
//     let media_id_clone = media_id;
//     let media_item = tauri::async_runtime::spawn_blocking(move || {
//         database::get_media_item_by_id(&get_media_conn, media_id_clone)
//             .map_err(|e| format!("Failed to get media item: {}", e))
//             .and_then(|opt| opt.ok_or_else(|| format!("Media item not found: {}", media_id_clone)))
//     }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//     // Only regenerate if the thumbnail doesn't exist or the file has been modified
//     if existing_thumbnail.is_none() || file_modified_time > media_item.updated_at {
//         // Generate a temporary thumbnail file name (this file will be created by ffmpeg)
//         let temp_thumbnail_path = format!("{}.thumb.jpg", path);
//
//         // Run ffmpeg sidecar to extract a single frame at 1 second into a thumbnail image
//         let ffmpeg_cmd = Command::new_sidecar("ffmpeg")
//             .map_err(|e| format!("Failed to create ffmpeg sidecar command: {}", e))?;
//
//         // Scale the thumbnail to the requested size
//         let scale_arg = format!("scale={}:-1", size);
//
//         let ffmpeg_status = ffmpeg_cmd
//             .args(&[
//                 "-y", "-ss", "00:00:01",
//                 "-i", path,
//                 "-frames:v", "1",
//                 "-q:v", "8", "-vf", &scale_arg,
//                 &temp_thumbnail_path,
//             ])
//             .status()
//             .map_err(|e| format!("Failed to run ffmpeg sidecar: {}", e))?;
//
//         if !ffmpeg_status.success() {
//             return Err(format!("ffmpeg sidecar failed with status: {:?}", ffmpeg_status));
//         }
//
//         // Read the thumbnail file into memory
//         let thumbnail_data = fs::read(&temp_thumbnail_path)
//             .map_err(|e| format!("Failed to read thumbnail file: {}", e))?;
//
//         // Clean up the temporary thumbnail file
//         let _ = fs::remove_file(&temp_thumbnail_path); // Ignore errors here
//
//         // Insert the thumbnail using spawn_blocking
//         let thumbnail = Thumbnail {
//             media_id,
//             size,
//             data: thumbnail_data.clone(),
//             mime_type: "image/jpeg".to_string(),
//         };
//
//         // Open a new connection for this operation
//         let insert_thumbnail_conn = rusqlite::Connection::open(&db_path)
//             .map_err(|e| format!("Failed to open database for thumbnail insert: {}", e))?;
//
//         let thumbnail_clone = thumbnail.clone();
//         tauri::async_runtime::spawn_blocking(move || {
//             database::insert_thumbnail(&insert_thumbnail_conn, &thumbnail_clone)
//                 .map_err(|e| format!("Failed to insert thumbnail: {}", e))
//         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//         // Update the media item's updated_at timestamp using spawn_blocking
//         // Open a new connection for this operation
//         let update_conn = rusqlite::Connection::open(&db_path)
//             .map_err(|e| format!("Failed to open database for update: {}", e))?;
//
//         let media_id_clone = media_id;
//         let file_modified_time_clone = file_modified_time;
//         tauri::async_runtime::spawn_blocking(move || {
//             update_conn.execute(
//                 "UPDATE media_items SET updated_at = ?1 WHERE id = ?2",
//                 rusqlite::params![file_modified_time_clone, media_id_clone],
//             ).map_err(|e| format!("Failed to update media item timestamp: {}", e))
//         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//         // Get the updated thumbnail using spawn_blocking
//         // Open a new connection for this operation
//         let get_thumbnail_conn = rusqlite::Connection::open(&db_path)
//             .map_err(|e| format!("Failed to open database for thumbnail retrieval: {}", e))?;
//
//         let media_id_clone = media_id;
//         let size_clone = size;
//         let thumbnail = tauri::async_runtime::spawn_blocking(move || {
//             database::get_thumbnail_by_media_id(&get_thumbnail_conn, media_id_clone, size_clone)
//                 .map_err(|e| format!("Failed to get thumbnail: {}", e))
//                 .and_then(|opt| opt.ok_or_else(|| format!("Thumbnail not found for media_id: {}", media_id_clone)))
//         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//         // Encode the data as base64
//         let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);
//
//         // Create a data URL
//         let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);
//
//         Ok(data_url)
//     } else {
//         // Thumbnail exists and file hasn't been modified, use existing thumbnail
//         println!("Using existing thumbnail for video {}", path);
//
//         let thumbnail = existing_thumbnail.unwrap();
//
//         // Encode the data as base64
//         let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);
//
//         // Create a data URL
//         let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);
//
//         Ok(data_url)
//     }
// }
//
// /// Generate a thumbnail for an image at a specific size
// pub async fn generate_image_thumbnail(
//     path: &str,
//     _size: i32, // Parameter kept for backward compatibility but not used
//     media_id: i64,
//     conn: Connection
// ) -> Result<String, String> {
//     // Use standard thumbnail size
//     let size = 256;
//
//     // Get file metadata to check modification time
//     let file_metadata = fs::metadata(path)
//         .map_err(|e| format!("Failed to get file metadata: {}", e))?;
//
//     // Get the file's modification time as a Unix timestamp
//     let file_modified_time = file_metadata.modified()
//         .map_err(|e| format!("Failed to get file modification time: {}", e))?
//         .duration_since(SystemTime::UNIX_EPOCH)
//         .map_err(|e| format!("Failed to calculate file modification time: {}", e))?
//         .as_secs() as i64;
//
//     // Get the database path for creating new connections
//     let db_path = state::get_db_path()?;
//
//     // Check if we need to regenerate the thumbnail using spawn_blocking
//     let media_id_clone = media_id;
//     let size_clone = size;
//     let existing_thumbnail = tauri::async_runtime::spawn_blocking(move || {
//         database::get_thumbnail_by_media_id(&conn, media_id_clone, size_clone)
//             .map_err(|e| format!("Database error: {}", e))
//     }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//     // Get the media item to check its updated_at timestamp using spawn_blocking
//     // Open a new connection for this operation
//     let get_media_conn = rusqlite::Connection::open(&db_path)
//         .map_err(|e| format!("Failed to open database for media item retrieval: {}", e))?;
//
//     let media_id_clone = media_id;
//     let media_item = tauri::async_runtime::spawn_blocking(move || {
//         database::get_media_item_by_id(&get_media_conn, media_id_clone)
//             .map_err(|e| format!("Failed to get media item: {}", e))
//             .and_then(|opt| opt.ok_or_else(|| format!("Media item not found: {}", media_id_clone)))
//     }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//     // Determine the MIME type based on file extension
//     let mime_type = if path.to_lowercase().ends_with(".jpg") || path.to_lowercase().ends_with(".jpeg") {
//         "image/jpeg"
//     } else if path.to_lowercase().ends_with(".png") {
//         "image/png"
//     } else if path.to_lowercase().ends_with(".gif") {
//         "image/gif"
//     } else {
//         "application/octet-stream" // Default MIME type
//     };
//
//     // Only regenerate if the thumbnail doesn't exist or the file has been modified
//     if existing_thumbnail.is_none() || file_modified_time > media_item.updated_at {
//         // Read the image file into memory
//         let image_data = fs::read(path)
//             .map_err(|e| format!("Failed to read image file: {}", e))?;
//
//         // For images, we'll use the same image as the thumbnail
//         // In a real implementation, you might want to resize the image to the requested size
//         let thumbnail_data = image_data.clone();
//
//         // Insert the thumbnail using spawn_blocking
//         let thumbnail = Thumbnail {
//             media_id,
//             size,
//             data: thumbnail_data.clone(),
//             mime_type: mime_type.to_string(),
//         };
//
//         // Open a new connection for this operation
//         let insert_thumbnail_conn = rusqlite::Connection::open(&db_path)
//             .map_err(|e| format!("Failed to open database for thumbnail insert: {}", e))?;
//
//         let thumbnail_clone = thumbnail.clone();
//         tauri::async_runtime::spawn_blocking(move || {
//             database::insert_thumbnail(&insert_thumbnail_conn, &thumbnail_clone)
//                 .map_err(|e| format!("Failed to insert thumbnail: {}", e))
//         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//         // Update the media item's updated_at timestamp using spawn_blocking
//         // Open a new connection for this operation
//         let update_conn = rusqlite::Connection::open(&db_path)
//             .map_err(|e| format!("Failed to open database for update: {}", e))?;
//
//         let media_id_clone = media_id;
//         let file_modified_time_clone = file_modified_time;
//         tauri::async_runtime::spawn_blocking(move || {
//             update_conn.execute(
//                 "UPDATE media_items SET updated_at = ?1 WHERE id = ?2",
//                 rusqlite::params![file_modified_time_clone, media_id_clone],
//             ).map_err(|e| format!("Failed to update media item timestamp: {}", e))
//         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//         // Get the updated thumbnail using spawn_blocking
//         // Open a new connection for this operation
//         let get_thumbnail_conn = rusqlite::Connection::open(&db_path)
//             .map_err(|e| format!("Failed to open database for thumbnail retrieval: {}", e))?;
//
//         let media_id_clone = media_id;
//         let size_clone = size;
//         let thumbnail = tauri::async_runtime::spawn_blocking(move || {
//             database::get_thumbnail_by_media_id(&get_thumbnail_conn, media_id_clone, size_clone)
//                 .map_err(|e| format!("Failed to get thumbnail: {}", e))
//                 .and_then(|opt| opt.ok_or_else(|| format!("Thumbnail not found for media_id: {}", media_id_clone)))
//         }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
//
//         // Encode the data as base64
//         let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);
//
//         // Create a data URL
//         let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);
//
//         Ok(data_url)
//     } else {
//         // Thumbnail exists and file hasn't been modified, use existing thumbnail
//         println!("Using existing thumbnail for image {}", path);
//
//         let thumbnail = existing_thumbnail.unwrap();
//
//         // Encode the data as base64
//         let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);
//
//         // Create a data URL
//         let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);
//
//         Ok(data_url)
//     }
// }