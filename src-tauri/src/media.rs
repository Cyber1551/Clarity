use std::fs;
use std::path::PathBuf;
use tauri::api::process::Command;
use base64::Engine;

use crate::models::{MediaMetadata, MediaItem, Thumbnail};
use crate::database;
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

    // Generate a temporary thumbnail file name (this file will be created by ffmpeg)
    let temp_thumbnail_path = format!("{}.thumb.jpg", path);

    // Run ffmpeg sidecar to extract a single frame at 1 second into a thumbnail image
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

    // Run ffprobe sidecar to extract the video duration
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
    let duration_str = ffprobe_output.stdout.trim().to_string();
    let duration: f64 = duration_str
        .parse()
        .map_err(|e| format!("Failed to parse duration: {}", e))?;
    let duration_sec = duration.round() as u32;

    // Read the thumbnail file into memory
    let thumbnail_data = fs::read(&temp_thumbnail_path)
        .map_err(|e| format!("Failed to read thumbnail file: {}", e))?;

    // Clean up the temporary thumbnail file
    let _ = fs::remove_file(&temp_thumbnail_path); // Ignore errors here

    // Check if the media item already exists in the database
    let (conn, media_id) = {
        let existing_media = database::get_media_item_by_path(&conn, path)
            .map_err(|e| format!("Database error: {}", e))?;

        if let Some(media) = existing_media {
            // Media item exists, update it
            (conn, media.id.unwrap())
        } else {
            // Media item doesn't exist, insert it
            let media_item = MediaItem {
                id: None,
                path: path.to_string(),
                title,
                media_type: "video".to_string(),
                length: Some(duration_sec as i64),
                created_at: database::get_current_timestamp(),
                updated_at: database::get_current_timestamp(),
            };

            let id = database::insert_media_item(&conn, &media_item)
                .map_err(|e| format!("Failed to insert media item: {}", e))?;
            (conn, id)
        }
    };

    // Insert or update the thumbnail
    let size = 32; // Default thumbnail size
    let thumbnail = Thumbnail {
        media_id,
        size,
        data: thumbnail_data.clone(),
        mime_type: "image/jpeg".to_string(),
    };

    database::insert_thumbnail(&conn, &thumbnail)
        .map_err(|e| format!("Failed to insert thumbnail: {}", e))?;

    // Encode the thumbnail data as base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail_data);
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
     size: i32,
     conn: Connection
 ) -> Result<MediaMetadata, String> {
     // Extract the filename from the path to use as the title
     let image_path = PathBuf::from(path);
     let title = image_path
         .file_name()
         .and_then(|name| name.to_str())
         .unwrap_or("Unknown")
         .to_string();

     // Check if the media item already exists in the database
     let (conn, media_id) = {
         let existing_media = database::get_media_item_by_path(&conn, path)
             .map_err(|e| format!("Database error: {}", e))?;

         if let Some(media) = existing_media {
             // Media item exists, update it
             (conn, media.id.unwrap())
         } else {
             // Media item doesn't exist, insert it
             let media_item = MediaItem {
                 id: None,
                 path: path.to_string(),
                 title,
                 media_type: "image".to_string(),
                 length: None, // Images don't have a duration
                 created_at: database::get_current_timestamp(),
                 updated_at: database::get_current_timestamp(),
             };

             let id = database::insert_media_item(&conn, &media_item)
                 .map_err(|e| format!("Failed to insert media item: {}", e))?;
             (conn, id)
         }
     };

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

     // Check if the thumbnail with this `media_id` and `size` exists
     let existing_thumbnail = database::get_thumbnail_by_media_id(&conn, media_id, size)
         .map_err(|e| format!("Database error: {}", e))?;

     // If the thumbnail doesn't exist, insert it
     if existing_thumbnail.is_none() {
         // Read the image file into memory
         let image_data = fs::read(path)
             .map_err(|e| format!("Failed to read image file: {}", e))?;

         // For images, we'll use the same image as the thumbnail
         let thumbnail_data = image_data.clone();

         let thumbnail = Thumbnail {
             media_id,
             size,
             data: thumbnail_data,
             mime_type: mime_type.to_string(),
         };

         database::insert_thumbnail(&conn, &thumbnail)
             .map_err(|e| format!("Failed to insert thumbnail: {}", e))?;
     }

     // Get the thumbnail data to encode as base64
     let thumbnail = database::get_thumbnail_by_media_id(&conn, media_id, size)
         .map_err(|e| format!("Failed to get thumbnail: {}", e))?
         .ok_or_else(|| format!("Thumbnail not found for media_id: {}", media_id))?;

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
pub async fn generate_video_thumbnail(
    path: &str,
    size: i32,
    media_id: i64,
    conn: Connection
) -> Result<String, String> {
    // Generate a temporary thumbnail file name (this file will be created by ffmpeg)
    let temp_thumbnail_path = format!("{}.thumb.jpg", path);

    // Run ffmpeg sidecar to extract a single frame at 1 second into a thumbnail image
    let ffmpeg_cmd = Command::new_sidecar("ffmpeg")
        .map_err(|e| format!("Failed to create ffmpeg sidecar command: {}", e))?;

    // Scale the thumbnail to the requested size
    let scale_arg = format!("scale={}:-1", size);

    let ffmpeg_status = ffmpeg_cmd
        .args(&[
            "-y", "-ss", "00:00:01",
            "-i", path,
            "-frames:v", "1",
            "-q:v", "8", "-vf", &scale_arg,
            &temp_thumbnail_path,
        ])
        .status()
        .map_err(|e| format!("Failed to run ffmpeg sidecar: {}", e))?;

    if !ffmpeg_status.success() {
        return Err(format!("ffmpeg sidecar failed with status: {:?}", ffmpeg_status));
    }

    // Read the thumbnail file into memory
    let thumbnail_data = fs::read(&temp_thumbnail_path)
        .map_err(|e| format!("Failed to read thumbnail file: {}", e))?;

    // Clean up the temporary thumbnail file
    let _ = fs::remove_file(&temp_thumbnail_path); // Ignore errors here

    // Insert the thumbnail
    let thumbnail = Thumbnail {
        media_id,
        size,
        data: thumbnail_data,
        mime_type: "image/jpeg".to_string(),
    };

    database::insert_thumbnail(&conn, &thumbnail)
        .map_err(|e| format!("Failed to insert thumbnail: {}", e))?;

    // Encode the data as base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);

    // Create a data URL
    let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);

    Ok(data_url)
}

/// Generate a thumbnail for an image at a specific size
pub async fn generate_image_thumbnail(
    path: &str,
    size: i32,
    media_id: i64,
    conn: Connection
) -> Result<String, String> {
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

    // Read the image file into memory
    let image_data = fs::read(path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;

    // For images, we'll use the same image as the thumbnail
    // In a real implementation, you might want to resize the image to the requested size
    let thumbnail_data = image_data.clone();

    // Insert the thumbnail
    let thumbnail = Thumbnail {
        media_id,
        size,
        data: thumbnail_data,
        mime_type: mime_type.to_string(),
    };

    database::insert_thumbnail(&conn, &thumbnail)
        .map_err(|e| format!("Failed to insert thumbnail: {}", e))?;

    // Encode the data as base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);

    // Create a data URL
    let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);

    Ok(data_url)
}
