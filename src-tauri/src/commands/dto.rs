use serde::Serialize;
use crate::media::{MediaType, MediaItem};
use crate::jobs::JobStatus;
use crate::media_links::MediaLinkRow;
use crate::thumbnails::ThumbnailRow;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDto {
    pub media_id: i64,
    pub rel_path: Option<String>,
    pub dir_path: Option<String>,
    pub file_name: Option<String>,
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
pub struct FileDto {
    pub id: i64,
    pub rel_path: String,
    pub dir_path: String,
    pub file_name: String,
    pub ext: String,
}

impl From<MediaLinkRow> for FileDto {
    fn from(link: MediaLinkRow) -> Self {
        Self {
            id: link.id,
            rel_path: link.rel_path,
            dir_path: link.dir_path,
            file_name: link.file_name,
            ext: link.ext,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDetailDto {
    pub media_id: i64,
    pub content_hash: String,
    pub media_type: MediaType,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i64>,
    pub quality_rating: i32,
    pub favorite_rating: i32,
    pub loved: bool,
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