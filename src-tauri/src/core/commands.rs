use tauri::State;
use crate::core::state;
use crate::core::state::AppState;
use crate::{database, media};
use crate::core::constants::BROKEN_THUMBNAIL;
use crate::types::MediaItemResponse;
use crate::cache::builder;

/// Initialize the database with a specified folder path
#[tauri::command]
pub async fn initialize_database(state: State<'_, AppState>, folder_path: String) -> Result<(), String> {
    println!("Initializing database...");
    let pool = state::initialize_database(&folder_path).await?;

    let mut db_lock = state.database_pool.lock().unwrap();
    *db_lock = Some(pool);

    Ok(())
}

/// Get media items
#[tauri::command]
pub async fn get_media_items(state: State<'_, AppState>) -> Result<Vec<MediaItemResponse>, String> {
    println!("Getting media items...");
    let pool = state.get_pool()?;

    let response_items = tauri::async_runtime::spawn_blocking(move || {
        let conn = state::get_connection(&pool)?;
        let media_items = database::media_items::get_all_media_items(&conn).map_err(|e| format!("Failed to get media items: {}", e))?;

        let mut responses = Vec::new();

        for media in media_items {
            //let tags = database::get_tags(&conn, media.id)?;
            //let bookmarks = database::get_bookmarks(&conn, media.id)?;

            let thumbnail_blob = database::thumbnails::get_thumbnail_by_media_id(&conn, &media.id).map_err(|e| format!("Failed to get thumbnail: {}", e))?;

            let base64 = match thumbnail_blob {
                Some(thumbnail) => media::helpers::generate_base64_from_image(thumbnail.data),
                None => media::helpers::generate_base64_from_image(BROKEN_THUMBNAIL.to_vec())
            };

            responses.push(MediaItemResponse {
                id: media.id,
                path: String::from(media.path.to_string_lossy()),
                file_name: media.file_name,
                file_size: media.file_size,
                file_extension: media.file_extension,
                media_type: media.media_type,
                video_length: media.video_length,
                thumbnail_base64: base64
            });
        }

        Ok::<_, String>(responses)
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
    
    Ok(response_items)
}

#[tauri::command]
pub async fn build_cache(state: State<'_, AppState>, folder_path: String) -> Result<(), String> {
    println!("Building cache...");
    let pool = state.get_pool()?;

    tauri::async_runtime::spawn_blocking(move || {
        let conn = state::get_connection(&pool)?;
        builder::build_cache(&conn, folder_path).map_err(|e| format!("Failed to build cache: {}", e))?;
        Ok::<_, String>(())
    }).await.map_err(|e| format!("Failed to spawn blocking task: {}", e))??;

    Ok(())
}