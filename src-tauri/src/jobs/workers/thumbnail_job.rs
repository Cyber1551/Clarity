use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use image::{GenericImageView, ImageFormat, ImageReader};
use rusqlite::Transaction;
use crate::core::constants::{FFMPEG_BIN, THUMBNAIL_SIZE};
use crate::core::error::{AppError, AppResult};
use crate::core::time::now_ms;
use crate::{db, filesystem};
use crate::db::thumbnails;
use crate::jobs::{JobEntry, JobStatus};
use crate::media::MediaType;

pub fn handle_thumbnail_job(tx: &Transaction, library_root: &Path, job: &JobEntry) -> AppResult<()> {
    let media_id_str = job.media_id.map(|id| id.to_string()).unwrap_or("none".to_string());
    println!("(job) handling thumbnail job id={} for media_id={:?}", job.id, media_id_str);

    let media_id = job.require_media_id()?;
    let media = match db::media::get_by_id(tx, &media_id)? {
        Some(m) => m,
        None => return Ok(())
    };

    // If thumbnail already ready, treat as idempotent: just delete job.
    if media.thumbnail_status == JobStatus::Done {
        return Ok(());
    }

    // Locate canonical file
    let canonical_path = filesystem::objects::find_canonical_objects_file(library_root, &media.content_hash)?;

    let thumb_result = generate_thumbnail(&canonical_path, media.media_type);
    let now = now_ms();

    match thumb_result {
        Ok(th) => {
            thumbnails::upsert_thumbnail(
                &tx,
                &media.content_hash,
                &th.data,
                th.width,
                th.height,
                now,
            )?;
            thumbnails::mark_thumbnail_done(&tx, media.id, now)?;
        }
        Err(e) => {
            eprintln!(
                "thumbnail job {} failed for media {}: {}",
                job.id, media.id, e
            );
            thumbnails::mark_thumbnail_error(&tx, media.id, now)?;
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
                "cannot generate thumbnail for unknown media type: {:?}",
                path
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
        .map_err(|e| AppError::Other(format!("encode webp thumbnail {:?}: {e}", path)))?;

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
        .arg("-vf").arg(format!("scale={}:-1", THUMBNAIL_SIZE))
        .arg("-f").arg("image2pipe")
        .arg("-vcodec").arg("libwebp")
        .arg("-lossless").arg("0")
        .arg("-quality").arg("60")
        .arg("-compression_level").arg("6")
        .arg("-") // pipe output to stdout
        .output()
        .map_err(|e| AppError::Other(format!("running ffmpeg for webp thumb on {:?}: {e}", path)))?;

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
            "ffmpeg returned empty webp thumbnail for {:?}",
            path
        )));
    }

    // Read dimensions from the generated WebP
    let img = image::load_from_memory(&webp_bytes)
        .map_err(|e| AppError::Other(format!("decode webp thumbnail for {:?}: {e}", path)))?;
    let (w, h) = img.dimensions();

    Ok(GeneratedThumbnail {
        data: webp_bytes,
        width: w as i64,
        height: h as i64,
    })
}