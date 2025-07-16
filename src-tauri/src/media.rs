use std::fs;
use std::path::PathBuf;
use tauri::api::process::Command;
use base64::Engine;

use crate::models::{VideoMetadata, MediaItem, Thumbnail};
use crate::database;
use rusqlite::Connection;

/// Extract metadata from a video file, including generating a thumbnail
pub async fn extract_video_metadata(
    path: &str,
    conn: Connection
) -> Result<VideoMetadata, String> {
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
    let thumbnail = Thumbnail {
        id: None,
        media_id,
        data: thumbnail_data,
        mime_type: "image/jpeg".to_string(),
    };

    let thumbnail_id = database::insert_thumbnail(&conn, &thumbnail)
        .map_err(|e| format!("Failed to insert thumbnail: {}", e))?;

    // Return the metadata with the database IDs
    Ok(VideoMetadata {
        id: media_id,
        thumbnail_id,
        duration: duration_sec,
    })
}

/// Extract metadata from an image file, including generating a thumbnail
pub async fn extract_image_metadata(
    path: &str,
    conn: Connection
) -> Result<VideoMetadata, String> {
    // Extract the filename from the path to use as the title
    let image_path = PathBuf::from(path);
    let title = image_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Read the image file into memory
    let image_data = fs::read(path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;

    // For images, we'll use the same image as the thumbnail
    // In a production app, you might want to resize the image to create a smaller thumbnail
    let thumbnail_data = image_data.clone();

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

    // Insert or update the thumbnail
    let thumbnail = Thumbnail {
        id: None,
        media_id,
        data: thumbnail_data,
        mime_type: mime_type.to_string(),
    };

    let thumbnail_id = database::insert_thumbnail(&conn, &thumbnail)
        .map_err(|e| format!("Failed to insert thumbnail: {}", e))?;

    // Return the metadata with the database IDs
    // For images, we set duration to 0 since they don't have a duration
    Ok(VideoMetadata {
        id: media_id,
        thumbnail_id,
        duration: 0,
    })
}

/// Get a thumbnail as a data URL
pub fn get_thumbnail_as_data_url(conn: Connection, thumbnail_id: i64) -> Result<String, String> {
    // Get the thumbnail directly by its ID
    let thumbnail = database::get_thumbnail_by_id(&conn, thumbnail_id)
        .map_err(|e| format!("Failed to get thumbnail: {}", e))?
        .ok_or_else(|| format!("Thumbnail not found: {}", thumbnail_id))?;

    // Encode the data as base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&thumbnail.data);

    // Create a data URL
    let data_url = format!("data:{};base64,{}", thumbnail.mime_type, base64_data);

    Ok(data_url)
}
