use serde::{Deserialize, Serialize};
use crate::media::{MediaType, MediaItem};
use crate::jobs::JobStatus;
use crate::media_files::MediaFileRow;
use crate::tags::TagRow;
use crate::thumbnails::ThumbnailRow;

/// Structured search request from the command palette. `text` drives the FTS title match; the rest are optional structured filters.
/// `quality`/`favorite` are minimums (>=); `tags` are slugs (AND).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub media_type: Option<MediaType>,
    #[serde(default)]
    pub quality: Option<i32>,
    #[serde(default)]
    pub favorite: Option<i32>,
    #[serde(default)]
    pub loved: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub reviewed: Option<bool>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDto {
    pub media_id: i64,
    pub rel_path: Option<String>,
    pub dir_path: Option<String>,
    pub file_name: Option<String>,
    pub display_name: Option<String>,
    pub ext: Option<String>,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub quality_rating: i32,
    pub favorite_rating: i32,
    pub loved: bool,
    pub hash_status: JobStatus,
    pub metadata_status: JobStatus,
    pub thumbnail_status: JobStatus,
    pub content_hash: String,
    pub reviewed_at: Option<i64>,
}

impl From<MediaItem> for MediaItemDto {
    fn from(item: MediaItem) -> Self {
        Self {
            media_id: item.media.id,
            rel_path: item.rel_path,
            dir_path: item.dir_path,
            file_name: item.file_name,
            display_name: item.media.display_name,
            ext: item.ext,
            media_type: item.media.media_type,
            width: item.media.width,
            height: item.media.height,
            duration_ms: item.media.duration_ms,
            quality_rating: item.media.quality_rating,
            favorite_rating: item.media.favorite_rating,
            loved: item.media.loved,
            hash_status: item.media.hash_status,
            metadata_status: item.media.metadata_status,
            thumbnail_status: item.media.thumbnail_status,
            content_hash: item.media.content_hash,
            reviewed_at: item.media.reviewed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDto {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

impl From<TagRow> for TagDto {
    fn from(tag: TagRow) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            slug: tag.slug,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusDto {
    pub dirty_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDto {
    pub id: i64,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
}

impl From<MediaFileRow> for FileDto {
    fn from(file: MediaFileRow) -> Self {
        Self {
            id: file.id,
            rel_path: file.rel_path,
            dir_path: file.dir_path,
            file_name: file.file_name,
            ext: file.ext,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDetailDto {
    pub media_id: i64,
    pub content_hash: String,
    pub media_type: MediaType,
    pub display_name: Option<String>,
    pub original_file_name: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub quality_rating: i32,
    pub favorite_rating: i32,
    pub loved: bool,
    pub size_bytes: i64,
    pub created_at: i64,
    pub reviewed_at: Option<i64>,
    pub tags: Vec<TagDto>,
    pub files: Vec<FileDto>,
    pub canonical_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailDto {
    pub blob: Vec<u8>,
    pub mimetype: String,
}

impl From<ThumbnailRow> for ThumbnailDto {
    fn from(thumbnail: ThumbnailRow) -> Self {
        Self {
            blob: thumbnail.thumbnail_blob,
            mimetype: thumbnail.mimetype,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSkippedItemDto {
    pub media_id: i64,
    pub content_hash: String,
    pub file_name: String,
    pub original_import_folder: Option<String>,
    pub original_rel_path: Option<String>,
    pub existing_dir_path: Option<String>,
    pub existing_rel_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResultDto {
    pub folder_name: String,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub skipped_items: Vec<ImportSkippedItemDto>,
}
