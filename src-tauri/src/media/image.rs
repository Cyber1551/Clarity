use std::fs;
use tauri::api::process::Command;
use crate::core::constants::{THUMBNAIL_EXTENSION, THUMBNAIL_SIZE};
use crate::media;

// Valid file extensions for images and videos
const VALID_IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg", "webp"];

/// Check if a file is an image based on its extension
pub fn is_image_file(path: &str) -> bool {
    let extension = media::helpers::get_extension(path);
    VALID_IMAGE_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

/// Generate a thumbnail for an image at a specific size
pub fn generate_image_thumbnail(path: &str) -> Result<Vec<u8>, String> {
    // Generate a temporary thumbnail file name (ffmpeg will create this file)
    let temp_thumbnail_path = format!("{}.thumb.{}", path, THUMBNAIL_EXTENSION);

    // Run ffmpeg sidecar to scale down the image into a thumbnail
    let ffmpeg_cmd = Command::new_sidecar("ffmpeg")
        .map_err(|e| format!("Failed to create ffmpeg sidecar command: {}", e))?;

    // Scale the thumbnail to the requested size
    let scale_arg = format!("scale={}:-1", THUMBNAIL_SIZE);

    let ffmpeg_status = ffmpeg_cmd
        .args(&[
            "-i", path,
            "-vf", &scale_arg,
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
    println!("Removed temporary thumbnail file: {}", temp_thumbnail_path);

    Ok(thumbnail_data.clone())
}