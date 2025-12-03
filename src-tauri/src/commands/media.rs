use serde::Serialize;
use crate::core::config;
use crate::core::constants::BROKEN_THUMBNAIL;
use crate::core::error::AppError;
use crate::db::schema::DbConn;
use crate::db;
use crate::jobs::JobStatus;
use crate::media::MediaType;

/// Data transfer object for media items sent to the frontend.
///
/// Combines file metadata, media information, and thumbnails with base64-encoded
/// thumbnail data URLs ready for direct use in HTML `<img>` tags.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDto {
    pub media_id: i64,
    pub file_id: i64,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub hash_status: JobStatus,
    pub metadata_status: JobStatus,
    pub thumbnail_status: JobStatus,
    pub content_hash: String,
    pub thumbnail_data_url: String,
}

/// Retrieves all media items from the library.
///
/// Thin command handler that delegates to the database layer and handles presentation logic.
#[tauri::command]
pub fn get_all_media(app: tauri::AppHandle) -> Result<Vec<MediaItemDto>, String> {
    let root = config::get_library_root(&app).map_err(|e: AppError| e.report())?;
    let mut conn = DbConn::new(&root).map_err(|e: AppError| e.report())?;
    let tx = DbConn::transaction(&mut conn).map_err(|e: AppError| e.report())?;

    // Delegate to database layer
    let rows = db::media::get_all_with_thumbnails(&tx).map_err(|e: AppError| e.report())?;

    // Convert database rows to DTOs with presentation logic (base64 encoding)
    let items: Vec<MediaItemDto> = rows
        .into_iter()
        .map(MediaItemDto::from_db_row)
        .collect();

    tx.commit().map_err(|e| e.to_string())?;

    Ok(items)
}

impl MediaItemDto {
    /// Converts a database row to a DTO with base64-encoded thumbnail data URL.
    ///
    /// Handles thumbnail encoding by converting the binary blob to a base64-encoded
    /// data URL. Falls back to the broken thumbnail icon if no thumbnail is available.
    fn from_db_row(row: db::media::MediaItemRow) -> Self {
        let thumbnail_data_url = match row.thumbnail_blob {
            Some(blob) => {
                let encoded = base64_encode(&blob);
                format!("data:image/webp;base64,{encoded}")
            }
            None => {
                let encoded = base64_encode(BROKEN_THUMBNAIL);
                format!("data:image/webp;base64,{encoded}")
            }
        };

        Self {
            media_id: row.media_id,
            file_id: row.file_id,
            rel_path: row.rel_path,
            dir_path: row.dir_path,
            file_name: row.file_name,
            ext: row.ext,
            media_type: row.media_type,
            width: row.width,
            height: row.height,
            duration_ms: row.duration_ms,
            hash_status: row.hash_status,
            metadata_status: row.metadata_status,
            thumbnail_status: row.thumbnail_status,
            content_hash: row.content_hash,
            thumbnail_data_url,
        }
    }
}

/// Encodes binary data to base64 string.
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}
