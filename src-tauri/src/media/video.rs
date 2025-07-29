use std::fs;
use tauri::api::process::Command;
use crate::app::constants::{THUMBNAIL_EXTENSION, THUMBNAIL_SIZE};
use crate::media;

const VALID_VIDEO_EXTENSIONS: [&str; 7] = ["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"];

/// Check if a file is a video based on its extension
pub fn is_video_file(path: &str) -> bool {
    let extension = media::helpers::get_extension(path);
    VALID_VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

pub fn get_video_duration(path: &str) -> Option<f64> {
    if !is_video_file(path) {
        return None;
    }

    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()
        .ok()?;

    output.stdout.trim().parse::<f64>().ok()
}

/// Generate a thumbnail for a video at a specific size
pub fn generate_video_thumbnail(path: &str) -> Result<Vec<u8>, String> {
    // Generate a temporary thumbnail file name (ffmpeg will create this file)
    let temp_thumbnail_path = format!("{}.thumb.{}", path, THUMBNAIL_EXTENSION);

    // Run ffmpeg sidecar to scale down the image into a thumbnail
    let ffmpeg_cmd = Command::new_sidecar("ffmpeg")
        .map_err(|e| format!("Failed to create ffmpeg sidecar command: {}", e))?;

    // Scale the thumbnail to the requested size
    let scale_arg = format!("scale={}:-1", THUMBNAIL_SIZE);

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

    Ok(thumbnail_data.clone())
}
