use crate::core::constants::{
    AXIS_BY_FAVORITE, AXIS_BY_RATING, AXIS_BY_TAG, AXIS_BY_TYPE, AXIS_LOVED, LIBRARY_DIRECTORY,
    TYPE_FOLDER_IMAGE, TYPE_FOLDER_OTHER, TYPE_FOLDER_VIDEO,
};
use crate::filesystem::path::sanitize_component;
use crate::media::{MediaRow, MediaType};
use crate::tags::TagRow;

pub fn type_folder(media_type: MediaType) -> &'static str {
    match media_type {
        MediaType::Image => TYPE_FOLDER_IMAGE,
        MediaType::Video => TYPE_FOLDER_VIDEO,
        MediaType::Unknown => TYPE_FOLDER_OTHER,
    }
}

/// The Library directories a reviewed item should be hardlinked into, one per active axis.
/// Rating/Favorite only project when >= 1; Loved is a single flat folder.
pub fn desired_dirs(media: &MediaRow, tags: &[TagRow]) -> Vec<String> {
    let mut dirs = Vec::new();

    dirs.push(format!(
        "{LIBRARY_DIRECTORY}/{AXIS_BY_TYPE}/{}",
        type_folder(media.media_type)
    ));

    if media.quality_rating >= 1 {
        dirs.push(format!(
            "{LIBRARY_DIRECTORY}/{AXIS_BY_RATING}/{}",
            media.quality_rating
        ));
    }

    if media.favorite_rating >= 1 {
        dirs.push(format!(
            "{LIBRARY_DIRECTORY}/{AXIS_BY_FAVORITE}/{}",
            media.favorite_rating
        ));
    }

    if media.loved {
        dirs.push(format!("{LIBRARY_DIRECTORY}/{AXIS_LOVED}"));
    }

    for tag in tags {
        dirs.push(format!("{LIBRARY_DIRECTORY}/{AXIS_BY_TAG}/{}", tag.slug));
    }

    dirs
}

/// The base file name (no extension) for this item's projected hardlinks.
pub fn base_name(media: &MediaRow) -> String {
    let raw = media.display_name.as_deref().unwrap_or("untitled");
    sanitize_component(raw)
}

pub fn join_name(stem: &str, ext: &str) -> String {
    if ext.is_empty() {
        stem.to_string()
    } else {
        format!("{stem}.{ext}")
    }
}

/// Disambiguates a colliding base name; the content hash is unique per item, so this is too.
pub fn collision_name(base: &str, ext: &str, content_hash: &str) -> String {
    let suffix = &content_hash[..content_hash.len().min(8)];
    join_name(&format!("{base}__{suffix}"), ext)
}
