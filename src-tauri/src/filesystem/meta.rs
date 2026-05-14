use std::path::Path;
use chrono::{DateTime, Utc};
use image::ImageReader;
use serde::Deserialize;
use tauri_plugin_shell::ShellExt;
use crate::core::constants::FFPROBE_BIN;
use crate::core::error::{AppError, AppResult};
use crate::core::app_handle;
use crate::media::MediaType;

pub struct ProbedMetadata {
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration_ms: Option<i64>, // None for images
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    duration: Option<String>, // ffprobe returns seconds I believe TODO: verify this
}

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
}

pub fn get_file_size(path: &Path) -> AppResult<i64> {
    let meta = std::fs::metadata(path)?;
    Ok(meta.len() as i64)
}

pub fn get_mtime(path: &Path) -> AppResult<i64> {
    let meta = std::fs::metadata(path)?;
    let dt: DateTime<Utc> = meta.modified()?.into();
    Ok(dt.timestamp_millis())
}

pub async fn probe_media_metadata(path: &Path, media_type: MediaType) -> AppResult<ProbedMetadata> {
    // For now, we just return None values and let the pipeline continue.
    // Could replace this with actual logic later if this causes a problem
    let probed_metadata = match media_type {
        MediaType::Image => probe_image_metadata(path)?,
        MediaType::Video => probe_video_metadata(path).await?,
        MediaType::Unknown => ProbedMetadata {
            width: None,
            height: None,
            duration_ms: None,
        }
    };

    Ok(probed_metadata)
}

fn probe_image_metadata(path: &Path) -> AppResult<ProbedMetadata> {
    // Just need dimensions; no need to fully decode to RGBA
    let img = ImageReader::open(path)
        .map_err(|e| AppError::Other(format!("open image for metadata {path:?}: {e}")))?
        .with_guessed_format()
        .map_err(|e| AppError::Other(format!("guess image format {path:?}: {e}")))?
        .into_dimensions()
        .map_err(|e| AppError::Other(format!("read image dimensions {path:?}: {e}")))?;

    let (w, h) = img;
    Ok(ProbedMetadata {
        width: Some(w as i64),
        height: Some(h as i64),
        duration_ms: None,
    })
}

async fn probe_video_metadata(path: &Path) -> AppResult<ProbedMetadata> {
    let app = app_handle::get_handle();
    let ffprobe = app.shell().sidecar(FFPROBE_BIN).map_err(|e| AppError::Other(format!("Failed to find ffprobe sidecar: {e}")))?;

    let output = ffprobe
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg("-select_streams")
        .arg("v:0")
        .arg(path)
        .output().await
        .map_err(|e| AppError::Other(format!("running ffprobe on {path:?}: {e}")))?;

    if !output.status.success() {
        return Err(AppError::Other(format!(
            "ffprobe failed on {:?}: {}",
            path,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::Other(format!("parse ffprobe JSON for {path:?}: {e}")))?;

    let stream = parsed
        .streams.first()
        .ok_or_else(|| AppError::Other(format!("no video stream in {path:?}")))?;

    let width = stream.width.map(|w| w as i64);
    let height = stream.height.map(|h| h as i64);

    let duration_ms = stream.duration.as_ref().and_then(|s| {
        s.parse::<f64>()
            .ok()
            .map(|secs| (secs * 1000.0).round() as i64)
    });

    Ok(ProbedMetadata {
        width,
        height,
        duration_ms,
    })
}