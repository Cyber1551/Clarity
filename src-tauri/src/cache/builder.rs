use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;
use crate::core::constants::{DB_NAME, UNSORTED_DIRECTORY};
use crate::database::schema::DbConn;
use crate::errors::{AppError, AppResult};
use crate::media::{file_utils, image_utils};
use crate::media::video_utils;

pub fn scan_unsorted(library_root: &Path) -> AppResult<()> {
    let _conn = DbConn::new(&library_root)?;
    let unsorted_dir = library_root.join(UNSORTED_DIRECTORY);

    let mut seen_paths = HashSet::<String>::new();

    for entry in WalkDir::new(&unsorted_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Skip the DB file
        if path.file_name() == Some(OsStr::new(DB_NAME)) {
            continue;
        }

        let rel_path = match path.strip_prefix(&library_root) {
            Ok(rel) => rel,
            Err(_) => return Err(AppError::InvalidMedia {
                path: path.to_path_buf()
            })
        };

        let rel_str = file_utils::path_to_utf8_string(&rel_path)?; // AppErrors::InvalidFileName
        seen_paths.insert(rel_str.clone());

        let is_image = image_utils::is_image_file(&path);
        let is_video = video_utils::is_video_file(&path);

        if !(is_image || is_video) {
            continue; // not a supported media file
        }

        // TODO: compute hash, width/height (images), duration (videos), etc.
        // TODO: insert into database (media/files/thumbnails)
    }

    println!("{:?}", seen_paths);

    Ok(())
}