use std::path::Path;
use crate::filesystem::path::get_extension;

const VALID_IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg", "webp"];

pub fn is_image_file(path: &Path) -> bool {
    let extension = get_extension(path);
    VALID_IMAGE_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}