use std::path::Path;
use crate::filesystem::path::get_extension;

const VALID_VIDEO_EXTENSIONS: [&str; 7] = ["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"];

pub fn is_video_file(path: &Path) -> bool {
    let extension = get_extension(path);
    VALID_VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}