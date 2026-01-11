use std::path::Path;
use tracing::error;
use crate::core::error::AppResult;
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::filesystem::meta;
use crate::jobs::{JobEntry, JobStatus};

use std::sync::Arc;
use crate::db::pool::DbManager;

pub fn handle_metadata_job(db_manager: Arc<DbManager>, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let media_id = job.require_media_id()?;
    let media = {
        let mut conn = db_manager.get_connection(library_root)?;
        let tx = conn.transaction()?;
        let m = db::media::get_by_id(&tx, media_id)?;
        tx.commit()?;
        m
    };
    
    let media = match media {
        Some(m) => m,
        None => return Ok(())
    };

    // If metadata already ready, treat as idempotent
    if media.metadata_status == JobStatus::Done {
        return Ok(());
    }

    // Locate canonical file in .objects
    let canonical_path = filesystem::objects::find_canonical_objects_file(library_root, &media.content_hash)?;

    // HEAVY WORK: Probe media metadata outside transaction and WITHOUT holding a connection
    let meta_result = meta::probe_media_metadata(&canonical_path, media.media_type);

    let now = now_ms();

    match meta_result {
        Ok(probed) => {
            let mut conn = db_manager.get_connection(library_root)?;
            let tx = conn.transaction()?;
            db::media::update_metadata(&tx, media.id, &probed, now)?;
            tx.commit()?;
        }
        Err(e) => {
            error!("Metadata job {} failed for media {}: {}", job.id, media.id, e);
            let mut conn = db_manager.get_connection(library_root)?;
            let tx = conn.transaction()?;
            db::media::mark_metadata_error(&tx, media.id, now)?;
            tx.commit()?;
            return Err(e);
        }
    }

    Ok(())
}
