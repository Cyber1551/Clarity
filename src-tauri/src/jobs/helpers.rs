use std::path::Path;
use rusqlite::Connection;
use crate::core::error::AppResult;
use crate::{db, filesystem};

pub fn cleanup_orphaned_media(conn: &Connection, library_root: &Path, old_media_id: Option<i64>) -> AppResult<()> {
    let Some(media_id) = old_media_id else { return Ok(()); };

    // DB part (atomic + idempotent)
    if let Some(content_hash) = db::media::delete_unreferenced_by_id(conn, media_id)? {
        // Filesystem part (best-effort; don’t fail the job on fs issues)
        if let Err(e) = filesystem::objects::remove_canonical_objects_file(library_root, &content_hash) {
            eprintln!("cleanup_orphaned_media: failed to remove canonical object for {content_hash}: {e}");
        }
    }

    Ok(())
}