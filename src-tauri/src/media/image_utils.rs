use std::path::Path;
use crate::core::constants::VALID_IMAGE_EXTENSIONS;
use crate::filesystem::path::get_extension;

pub fn is_image_file(path: &Path) -> bool {
    let extension = get_extension(path);
    VALID_IMAGE_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}