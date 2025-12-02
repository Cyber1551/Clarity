use std::path::Path;
use crate::core::constants::VALID_VIDEO_EXTENSIONS;
use crate::filesystem::path::get_extension;

pub fn is_video_file(path: &Path) -> bool {
    let extension = get_extension(path);
    VALID_VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}