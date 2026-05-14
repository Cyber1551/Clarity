// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use app::commands::{library, media, import, logging as cmd_logging};
use app::core::config;
use app::core::logging;
use app::jobs::runner::JobWorkerManager;
use app::db::pool::DbManager;
use app::core::state::LibraryRootState;

use std::sync::Arc;

fn main() {
    // Initialize logging
    logging::init_logging();

    let db_manager = Arc::new(DbManager::new());
    let library_root_state = Arc::new(LibraryRootState(std::sync::Mutex::new(None)));

    tauri::Builder::default()
        .manage(JobWorkerManager::new())
        .manage(db_manager.clone())
        .manage(library_root_state.clone())
        .setup(move |app| {
            let handle = app.handle();
            app::core::app_handle::set_handle(handle.clone());
            let job_manager = app.state::<JobWorkerManager>();

            // Set app handles for event emission
            job_manager.set_app_handle(handle.clone());

            // Try to read existing library_root from config
            match config::get_library_root(handle) {
                Ok(root) => {
                    *app.state::<Arc<LibraryRootState>>().0.lock().unwrap() = Some(root.clone());
                    job_manager.try_start_worker(&root, db_manager.clone());
                }
                Err(e) => {
                    // First-run case: the user hasn't chosen a library yet.
                    // We'll start workers later when they pick one.
                    tracing::info!("Job worker not started (library root not set): {}", e);
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| if let tauri::WindowEvent::CloseRequested { .. } = event {
            // Shutdown threads cleanly on application close
            let job_manager = window.state::<JobWorkerManager>();
            job_manager.shutdown();
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            library::get_app_config,
            library::choose_library_root,
            library::initialize_library,
            library::open_library_root,
            library::restart_workers,
            media::get_media_items,
            media::get_media_item_by_rel_path,
            media::get_media_detail,
            media::get_thumbnail,
            media::mark_as_reviewed,
            media::review_and_promote,
            media::update_quality_rating,
            media::update_favorite_rating,
            media::toggle_loved,
            media::rename_media_file,
            import::import_files,
            import::get_import_folders,
            import::get_items_in_import_folder,
            cmd_logging::log_event,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
