use std::io::Cursor;
use std::path::{Path, PathBuf};
use image::{GenericImageView, ImageFormat, ImageReader};
use tracing::error;
use crate::core::constants::{FFMPEG_BIN, THUMBNAIL_SIZE};
use crate::core::error::{AppError, AppResult};
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::db::thumbnails;
use crate::jobs::{JobRow, JobStatus};
use crate::core::app_handle;
use crate::media::MediaType;

use std::sync::Arc;
use tauri_plugin_shell::ShellExt;
use uuid::Uuid;
use crate::db::pool::DbManager;
use crate::thumbnails::ThumbnailRow;

pub async fn handle_thumbnail_job(db_manager: Arc<DbManager>, library_root: &Path, job: &JobRow) -> AppResult<()> {
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

    // If thumbnail already ready, verify blob exists in DB
    if media.thumbnail_status == JobStatus::Done {
        let conn = db_manager.get_connection(library_root)?;
        if thumbnails::get_blob(&conn, &media.content_hash)?.is_some() {
            return Ok(());
        }
        // If status is done but blob missing, we fall through and regenerate it
    }

    // Locate canonical file
    let canonical_path = filesystem::objects::find_canonical_objects_file(library_root, &media.content_hash)?;

    // HEAVY WORK: generate thumbnail outside transaction and WITHOUT holding a connection
    let thumb_result = generate_thumbnail(&canonical_path, media.content_hash, media.media_type).await;
    let now = now_ms();

    match thumb_result {
        Ok(thumbnail_row) => {
            let mut conn = db_manager.get_connection(library_root)?;
            let tx = conn.transaction()?;
            thumbnails::upsert(&tx, thumbnail_row, now)?;
            db::media::mark_thumbnail_done(&tx, media.id, now)?;
            tx.commit()?;
        }
        Err(e) => {
            error!("Thumbnail job {} failed for media {}: {}", job.id, media.id, e);
            let mut conn = db_manager.get_connection(library_root)?;
            let tx = conn.transaction()?;
            db::media::mark_thumbnail_error(&tx, media.id, now)?;
            tx.commit()?;
            return Err(e);
        }
    }

    Ok(())
}

async fn generate_thumbnail(path: &Path, content_hash: String, media_type: MediaType) -> AppResult<ThumbnailRow> {
    match media_type {
        MediaType::Image => generate_image_thumbnail(path, content_hash),
        MediaType::Video => generate_video_thumbnail(path, content_hash).await,
        MediaType::Unknown => {
            Err(AppError::Other(format!(
                "cannot generate thumbnail for unknown media type: {path:?}"
            )))
        }
    }
}

fn generate_image_thumbnail(path: &Path, content_hash: String) -> AppResult<ThumbnailRow> {
    // Load and decode image
    let img = ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?; // AppError::ImageError

    let thumb = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let (w, h) = thumb.dimensions();

    let mut buf = Vec::new();

    // Encode as WebP via image crate
    thumb.write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
        .map_err(|e| AppError::Other(format!("encode webp thumbnail {path:?}: {e}")))?;

    Ok(ThumbnailRow {
        content_hash,
        thumbnail_blob: buf,
        mimetype: "image/webp".to_string(),
        width: w as i32,
        height: h as i32,
    })
}

async fn generate_video_thumbnail(path: &Path, content_hash: String) -> AppResult<ThumbnailRow> {
    let app = app_handle::get_handle();
    let ffmpeg = app.shell().sidecar(FFMPEG_BIN).map_err(|e| AppError::Other(format!("Failed to find ffmpeg sidecar: {e}")))?;

    let out_path: PathBuf = std::env::temp_dir()
        .join(format!("thumb-{}.jpg", Uuid::new_v4()));

    let output = ffmpeg
        .arg("-y")
        .arg("-loglevel").arg("error")
        .arg("-i").arg(path)
        .arg("-vf").arg(format!("select='if(gt(scene,0.4),1,between(t,3,3.05))',scale={THUMBNAIL_SIZE}:-2:flags=lanczos,format=yuvj420p"))
        .arg("-frames:v").arg("1")
        .arg("-vcodec").arg("mjpeg")
        .arg("-q:v").arg("4")
        .arg("-an")
        .arg("-map_metadata").arg("-1")
        .arg(&out_path)
        .output().await
        .map_err(|e| AppError::Other(format!("running ffmpeg for jpeg thumb on {path:?}: {e}")))?;

    if !output.status.success() {
        return Err(AppError::Other(format!(
            "ffmpeg jpeg thumbnail failed for {:?}: {}",
            path,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let bytes = tokio::fs::read(&out_path).await
        .map_err(|e| AppError::Other(format!("read thumbnail {:?}: {e}", out_path)))?;

    // Best-effort cleanup (don’t fail the request if cleanup fails)
    let _ = tokio::fs::remove_file(&out_path).await;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| AppError::Other(format!("decode jpeg thumbnail for {path:?}: {e}")))?;
    let (w, h) = img.dimensions();

    Ok(ThumbnailRow {
        content_hash,
        thumbnail_blob: bytes,
        mimetype: "image/jpeg".to_string(),
        width: w as i32,
        height: h as i32,
    })
}