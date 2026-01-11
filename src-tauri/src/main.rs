// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use app::commands::{library, media};
use app::core::config;
use app::core::logging;
use app::jobs::runner::JobWorkerManager;
use app::filesystem::watcher::FileWatcherManager;
use app::db::pool::DbManager;
use app::core::state::LibraryRootState;
use app::core::constants::BROKEN_THUMBNAIL;

use std::sync::Arc;

fn main() {
    // Initialize logging
    logging::init_logging();

    let db_manager = Arc::new(DbManager::new());
    let library_root_state = Arc::new(LibraryRootState(std::sync::Mutex::new(None)));

    tauri::Builder::default()
        .manage(JobWorkerManager::new())
        .manage(FileWatcherManager::new())
        .manage(db_manager.clone())
        .manage(library_root_state.clone())
        .setup(move |app| {
            let handle = app.handle();
            let job_manager = app.state::<JobWorkerManager>();
            let watcher_manager = app.state::<FileWatcherManager>();

            // Set app handles for event emission
            job_manager.set_app_handle(handle.clone());
            watcher_manager.set_app_handle(handle.clone());

            // Try to read existing library_root from config
            match config::get_library_root(handle) {
                Ok(root) => {
                    *app.state::<Arc<LibraryRootState>>().0.lock().unwrap() = Some(root.clone());
                    job_manager.try_start_worker(&root, db_manager.clone());
                    watcher_manager.try_start_watcher(&root);
                }
                Err(e) => {
                    // First-run case: the user hasn't chosen a library yet.
                    // We'll start workers and watchers later when they pick one.
                    tracing::info!("Job worker and file watcher not started (library root not set): {}", e);
                }
            }
            Ok(())
        })
        .register_uri_scheme_protocol("thumbnail", |ctx, request| {
            let hash = request.uri().path().trim_start_matches('/').to_lowercase();
            let app = ctx.app_handle();
            
            let res = (|| {
                let root = {
                    let state = app.state::<Arc<LibraryRootState>>();
                    let root_lock = state.0.lock().unwrap();
                    root_lock.as_ref().ok_or("Library root not set")?.clone()
                };
                let db_manager = app.state::<Arc<DbManager>>();
                let conn = db_manager.get_connection(&root)?;
                let blob = app::db::thumbnails::get_blob(&conn, &hash)?;
                if blob.is_none() {
                    tracing::warn!("Thumbnail NOT FOUND in DB for hash: {}", hash);
                }
                Ok::<_, Box<dyn std::error::Error>>(blob)
            })();

            match res {
                Ok(Some(blob)) => {
                    tauri::http::Response::builder()
                        .header("Content-Type", "image/webp")
                        .header("Cache-Control", "public, max-age=31536000, immutable")
                        .body(blob)
                        .unwrap()
                }
                Ok(None) => {
                    tauri::http::Response::builder()
                        .header("Content-Type", "image/webp")
                        .header("Cache-Control", "no-store, must-revalidate")
                        .body(BROKEN_THUMBNAIL.to_vec())
                        .unwrap()
                }
                Err(e) => {
                    tracing::error!("Thumbnail protocol error for hash {}: {}", hash, e);
                    tauri::http::Response::builder()
                        .header("Content-Type", "image/webp")
                        .header("Cache-Control", "no-store, must-revalidate")
                        .body(BROKEN_THUMBNAIL.to_vec())
                        .unwrap()
                }
            }
        })
        .on_window_event(|window, event| if let tauri::WindowEvent::CloseRequested { .. } = event {
            // Shutdown threads cleanly on application close
            let job_manager = window.state::<JobWorkerManager>();
            let watcher_manager = window.state::<FileWatcherManager>();
            job_manager.shutdown();
            watcher_manager.shutdown();
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            library::get_app_config,
            library::choose_library_root,
            library::initialize_library,
            media::get_all_media,
            media::get_media_feed,
            media::get_media_detail,
            media::list_tags,
            media::create_tag,
            media::tag_media,
            media::untag_media,
            media::mark_media_reviewed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
