use std::path::Path;
use rusqlite::Transaction;
use tracing::error;
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::filesystem::meta;
use crate::jobs::{JobEntry, JobStatus};

pub fn handle_metadata_job(tx: &Transaction, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let media_id = job.require_media_id()?;
    let media = match db::media::get_by_id(tx, media_id)? {
        Some(m) => m,
        None => return Ok(())
    };

    // If metadata already ready, treat as idempotent
    if media.metadata_status == JobStatus::Done {
        return Ok(());
    }

    // Locate canonical file in .objects
    let canonical_path = filesystem::objects::find_canonical_objects_file(library_root, &media.content_hash)?;

    // Probe media metadata
    let meta_result = meta::probe_media_metadata(&canonical_path, media.media_type);

    let now = now_ms();

    match meta_result {
        Ok(probed) => {
            db::media::update_metadata(tx, media.id, &probed, now)?;
        }
        Err(e) => {
            error!("Metadata job {} failed for media {}: {}", job.id, media.id, e);
            db::media::mark_metadata_error(tx, media.id, now)?;
            return Err(e);
        }
    }

    Ok(())
}
