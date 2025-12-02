use std::path::Path;
use rusqlite::Transaction;
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::filesystem::meta;
use crate::jobs::{JobEntry, JobStatus};

pub fn handle_metadata_job(tx: &Transaction, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let media_id_str = job.media_id.map(|id| id.to_string()).unwrap_or("none".to_string());
    println!("(job) handling metadata job id={} for media_id={:?}", job.id, media_id_str);

    let media_id = job.require_media_id()?;
    let media = match db::media::get_by_id(tx, &media_id)? {
        Some(m) => m,
        None => return Ok(())
    };

    // If metadata already ready, treat as idempotent: just delete job.
    if media.metadata_status == JobStatus::Done {
        return Ok(());
    }

    // Locate canonical file in .objects
    let canonical_path = filesystem::objects::find_canonical_objects_file(library_root, &media.content_hash)?;

    // Probe using whatever backend you choose
    let meta_result = meta::probe_media_metadata(&canonical_path, media.media_type);

    let now = now_ms();

    match meta_result {
        Ok(probed) => {
            db::media::update_media_metadata(&tx, media.id, &probed, now)?;
        }
        Err(e) => {
            eprintln!(
                "metadata job {} failed for media {}: {}",
                job.id, media.id, e
            );
            db::media::mark_metadata_error(&tx, media.id, now)?;
            return Err(e);
        }
    }

    Ok(())
}
