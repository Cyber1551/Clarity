use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use base64::Engine;
use crate::app::constants::THUMBNAIL_EXTENSION;
use crate::media::image;
use crate::media::video;
use crate::database::models::MediaItem;
use crate::utils;

// Directories to skip during scanning
const SKIP_DIRECTORIES: [&str; 4] = [".thumbnails", "node_modules", ".git", ".vscode"];

pub fn get_extension(path: &str) -> &str {
    Path::new(path).extension().and_then(OsStr::to_str).unwrap_or("unknown")
}

/// Check if a directory should be skipped during scanning
fn should_skip_directory(path: &str) -> bool {
    SKIP_DIRECTORIES.iter().any(|dir| path.contains(dir))
}

pub fn get_file_size(path: &str) -> u64 {
    match fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    }
}


pub fn generate_base64_from_image(image: Vec<u8>) -> String {
    // Convert to base64
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&image);

    // Create a data URL
    format!("data:image/{};base64,{}", THUMBNAIL_EXTENSION, base64_data)
}

/// Create a MediaItem from a file path
fn create_media_item(id: i64, path: &str) -> Option<MediaItem> {
    let path_obj = Path::new(path);
    let file_name = path_obj.file_name()?.to_str()?;

    let is_image = image::is_image_file(path);
    let is_video = video::is_video_file(path);

    if !is_image && !is_video {
        return None;
    }

    let file_extension = get_extension(path);
    let file_size = get_file_size(path);

    let media_type = if is_image { "image" } else { "video" };
    let length = video::get_video_duration(path);
    let now = utils::get_current_timestamp();

    Some(MediaItem {
        id,
        path: path.to_string(),
        file_name: file_name.to_string(),
        file_size,
        file_extension: file_extension.to_string(),
        media_type: media_type.to_string(),
        video_length: length,
        created_at: now,
        updated_at: now,
    })
}