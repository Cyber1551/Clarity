// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use app::commands::{library, media};
use app::core::config;
use app::core::logging;
use app::jobs::runner::JobWorkerManager;
use app::filesystem::watcher::FileWatcherManager;

fn main() {
    // Initialize logging
    logging::init_logging();

    tauri::Builder::default()
        .manage(JobWorkerManager::new())
        .manage(FileWatcherManager::new())
        .setup(|app| {
            let handle = app.handle();
            let job_manager = app.state::<JobWorkerManager>();
            let watcher_manager = app.state::<FileWatcherManager>();

            // Set app handles for event emission
            job_manager.set_app_handle(handle.clone());
            watcher_manager.set_app_handle(handle.clone());

            // Try to read existing library_root from config
            match config::get_library_root(handle) {
                Ok(root) => {
                    job_manager.try_start_worker(&root);
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
            media::get_all_media
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
