use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

use crate::core::constants::{DB_NAME, SORTED_DIRECTORY, UNSORTED_DIRECTORY};
use crate::database::schema::DbConn;
use crate::errors::AppResult;
use crate::media::{directory_utils, image_utils, video_utils};

pub fn initial_scan(root: &Path) -> AppResult<()> {
    directory_utils::ensure_core_dirs(root)?;

    let _conn = DbConn::new(root);

    let unsorted = root.join(UNSORTED_DIRECTORY);
    let sorted = root.join(SORTED_DIRECTORY);

    for scan_root in [unsorted, sorted] {
        if !scan_root.exists() {
            continue;
        }

        for entry in WalkDir::new(&scan_root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Skip the DB file
            if path.file_name() == Some(OsStr::new(DB_NAME)) {
                continue;
            }

            let path_str = path.to_string_lossy();

            let is_image = image_utils::is_image_file(&path_str);
            let is_video = video_utils::is_video_file(&path_str);

            if !(is_image || is_video) {
                continue; // not a supported media file
            }

            // TODO: compute hash, width/height (images), duration (videos), etc.
            // TODO: insert into database (media/files/thumbnails)
        }
    }

    Ok(())
}