use std::path::Path;
use crate::media::file_utils;

const VALID_VIDEO_EXTENSIONS: [&str; 7] = ["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"];

pub fn is_video_file(path: &Path) -> bool {
    let extension = file_utils::get_extension(path);
    VALID_VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}
