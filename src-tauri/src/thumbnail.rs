use std::fs;
use std::time::SystemTime;
use base64::Engine;
use rusqlite::Connection;
use tauri::api::process::Command;
use crate::{database, state};
use crate::constants::THUMBNAIL_EXTENSION;
use crate::image_helpers::generate_image_thumbnail;
use crate::media_helpers::{is_image_file, is_video_file};
use crate::models::Thumbnail;
use crate::state::get_connection;
use crate::video_helpers::generate_video_thumbnail;

pub async fn save_thumbnail(media_id: i64) -> Result<(), String> {
    let conn = get_connection()?;
    let item = database::get_media_item_by_id(&conn, media_id).map_err(|e| e.to_string())?;

    if item.is_none() {
        return Err(format!("Media Item {media_id} does not exist"));
    }

    if let Some(media_item) = item {
        let is_image = is_image_file(&media_item.path);
        let is_video = is_video_file(&media_item.path);

        let thumbnail_blob: Vec<u8> = if is_image {
            generate_image_thumbnail(&media_item.path).map_err(|e| e.to_string())?
        } else if is_video {
            generate_video_thumbnail(&media_item.path).map_err(|e| e.to_string())?
        } else {
            return Err(format!("Media Item {media_id} is not an image or video"));
        };

        let thumbnail = Thumbnail {
            media_id,
            data: thumbnail_blob,
            mime_type: String::from(THUMBNAIL_EXTENSION)
        };

        database::insert_thumbnail(&conn, &thumbnail).map_err(|e| e.to_string())?;
        return Ok(());
    }

    Err(String::from("Unknown Error"))
}

