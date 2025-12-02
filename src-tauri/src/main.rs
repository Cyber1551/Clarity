// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use app::commands::{library};
use app::core::config;
use app::core::logging;
use app::jobs::runner::JobWorkerManager;

fn main() {
    // Initialize logging
    logging::init_logging();

    tauri::Builder::default()
        .manage(JobWorkerManager::new())
        .setup(|app| {
            let handle = app.handle();
            let manager = app.state::<JobWorkerManager>();

            // Try to read existing library_root from config
            match config::get_library_root(handle) {
                Ok(root) => {
                    manager.try_start_worker(&root);
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
            // Shutdown the worker thread cleanly on application close
            let manager = window.state::<JobWorkerManager>();
            manager.shutdown();
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            library::get_app_config,
            library::choose_library_root,
            library::initialize_library
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
