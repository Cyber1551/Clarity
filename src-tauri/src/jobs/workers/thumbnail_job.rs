use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use image::{GenericImageView, ImageFormat, ImageReader};
use tracing::error;
use crate::core::constants::{FFMPEG_BIN, THUMBNAIL_SIZE};
use crate::core::error::{AppError, AppResult};
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::db::thumbnails;
use crate::jobs::{JobRow, JobStatus};
use crate::media::MediaType;

use std::sync::Arc;
use crate::db::pool::DbManager;

pub fn handle_thumbnail_job(db_manager: Arc<DbManager>, library_root: &Path, job: &JobRow) -> AppResult<()> {
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
    let thumb_result = generate_thumbnail(&canonical_path, media.media_type);
    let now = now_ms();

    match thumb_result {
        Ok(th) => {
            let mut conn = db_manager.get_connection(library_root)?;
            let tx = conn.transaction()?;
            thumbnails::upsert(
                &tx,
                &media.content_hash,
                &th.data,
                th.width,
                th.height,
                now,
            )?;
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

pub struct GeneratedThumbnail {
    pub data: Vec<u8>,
    pub width: i64,
    pub height: i64,
}

fn generate_thumbnail(path: &Path, media_type: MediaType) -> AppResult<GeneratedThumbnail> {
    match media_type {
        MediaType::Image => generate_image_thumbnail(path),
        MediaType::Video => generate_video_thumbnail(path),
        MediaType::Unknown => {
            // Best effort: try as image first, then as video, or just bail.
            // For now, bail with error so you see it in logs.
            Err(AppError::Other(format!(
                "cannot generate thumbnail for unknown media type: {path:?}"
            )))
        }
    }
}

fn generate_image_thumbnail(path: &Path) -> AppResult<GeneratedThumbnail> {
    // Load and decode image
    let img = ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?; // AppError::ImageError

    let thumb = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let (w, h) = thumb.dimensions();

    let mut buf = Vec::new();

    // Encode as WebP via image crate
    thumb
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
        .map_err(|e| AppError::Other(format!("encode webp thumbnail {path:?}: {e}")))?;

    Ok(GeneratedThumbnail {
        data: buf,
        width: w as i64,
        height: h as i64,
    })
}

fn generate_video_thumbnail(path: &Path) -> AppResult<GeneratedThumbnail> {
    let output = Command::new(Path::new(FFMPEG_BIN))
        .arg("-y")
        .arg("-ss").arg("1.0")
        .arg("-i").arg(path)
        .arg("-frames:v").arg("1")
        .arg("-vf").arg(format!("scale={THUMBNAIL_SIZE}:-1"))
        .arg("-f").arg("image2pipe")
        .arg("-vcodec").arg("libwebp")
        .arg("-lossless").arg("0")
        .arg("-quality").arg("60")
        .arg("-compression_level").arg("6")
        .arg("-") // pipe output to stdout
        .output()
        .map_err(|e| AppError::Other(format!("running ffmpeg for webp thumb on {path:?}: {e}")))?;

    if !output.status.success() {
        return Err(AppError::Other(format!(
            "ffmpeg webp thumbnail failed for {:?}: {}",
            path,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let webp_bytes = output.stdout;
    if webp_bytes.is_empty() {
        return Err(AppError::Other(format!(
            "ffmpeg returned empty webp thumbnail for {path:?}"
        )));
    }

    // Read dimensions from the generated WebP
    let img = image::load_from_memory(&webp_bytes)
        .map_err(|e| AppError::Other(format!("decode webp thumbnail for {path:?}: {e}")))?;
    let (w, h) = img.dimensions();

    Ok(GeneratedThumbnail {
        data: webp_bytes,
        width: w as i64,
        height: h as i64,
    })
}