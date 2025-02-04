// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use tauri::api::process::Command; // Tauri's async process command
use serde::Serialize;
use md5::compute;
use tauri::{Builder, http::ResponseBuilder};

#[derive(Serialize)]
pub struct VideoMetadata {
  /// Instead of a long base64 string, we now store the path to the cached thumbnail.
  pub thumbnail_path: String,
  pub duration: u32,     // Duration in seconds
}

#[tauri::command]
async fn extract_video_metadata(path: String) -> Result<VideoMetadata, String> {
  // Determine the directory of the video file.
  let video_path = PathBuf::from(&path);
  let video_dir = video_path
    .parent()
    .ok_or("Invalid video path; cannot determine parent directory")?;

  // Create a cache directory inside the video folder (e.g. ".thumbnails").
  let cache_dir = video_dir.join(".thumbnails");
  fs::create_dir_all(&cache_dir)
    .map_err(|e| format!("Failed to create cache directory: {}", e))?;

  // Generate a temporary thumbnail file name (this file will be created by ffmpeg).
  let temp_thumbnail_path = format!("{}.thumb.jpg", &path);

  // Run ffmpeg sidecar to extract a single frame at 1 second into a thumbnail image.
  let ffmpeg_cmd = Command::new_sidecar("ffmpeg")
    .map_err(|e| format!("Failed to create ffmpeg sidecar command: {}", e))?;
  let ffmpeg_status = ffmpeg_cmd
    .args(&[
      "-y", "-ss", "00:00:01",
      "-i", &path,
      "-frames:v", "1",
      "-q:v", "8", "-vf", "scale=320:-1",
      &temp_thumbnail_path,
    ])
    .status()
    .map_err(|e| format!("Failed to run ffmpeg sidecar: {}", e))?;
  if !ffmpeg_status.success() {
    return Err(format!("ffmpeg sidecar failed with status: {:?}", ffmpeg_status));
  }

  // Run ffprobe sidecar to extract the video duration.
  let ffprobe_cmd = Command::new_sidecar("ffprobe")
    .map_err(|e| format!("Failed to create ffprobe sidecar command: {}", e))?;
  let ffprobe_output = ffprobe_cmd
    .args(&[
      "-v", "error",
      "-select_streams", "v:0",
      "-show_entries", "format=duration",
      "-of", "default=noprint_wrappers=1:nokey=1",
      &path,
    ])
    .output()
    .map_err(|e| format!("Failed to run ffprobe sidecar: {}", e))?;
  if !ffprobe_output.status.success() {
    return Err(format!("ffprobe sidecar failed with status: {:?}", ffprobe_output.status));
  }
  // Since ffprobe_output.stdout is a String, trim it directly.
  let duration_str = ffprobe_output.stdout.trim().to_string();
  let duration: f64 = duration_str
    .parse()
    .map_err(|e| format!("Failed to parse duration: {}", e))?;
  let duration_sec = duration.round() as u32;

  // Instead of reading the thumbnail into a base64 string, we now move it to our cache directory.
  // Use a hash of the video path to generate a unique thumbnail filename.
  let thumb_filename = format!("{:x}.jpg", md5::compute(&path));
  let cached_thumbnail_path = cache_dir.join(thumb_filename);

  // Move the temporary thumbnail file into the cache directory.
  fs::rename(&temp_thumbnail_path, &cached_thumbnail_path)
    .map_err(|e| format!("Failed to move thumbnail to cache: {}", e))?;

  // Return the metadata with the thumbnail file path.
  Ok(VideoMetadata {
    thumbnail_path: cached_thumbnail_path.to_string_lossy().into_owned(),
    duration: duration_sec,
  })
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs_watch::init())
     .register_uri_scheme_protocol("newuri", move |_app_handle, request| {
                // Extract the requested path from the URI
                if let Some(path) = request.uri().strip_prefix("newuri://") {
                    // Construct the full path to the .thumbnails directory
                    let mut file_path = PathBuf::from(".thumbnails");
                    file_path.push(path);

                    // Read the file contents
                    match fs::read(&file_path) {
                        Ok(contents) => {
                            // Determine the MIME type (assuming PNG images here)
                            let mime_type = "image/png";
                            ResponseBuilder::new()
                                .mimetype(mime_type)
                                .status(200)
                                .body(contents)
                        }
                        Err(_) => ResponseBuilder::new().status(404).body(Vec::new()),
                    }
                } else {
                    ResponseBuilder::new().status(400).body(Vec::new())
                }
            })
    .invoke_handler (tauri::generate_handler![
        extract_video_metadata
     ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
