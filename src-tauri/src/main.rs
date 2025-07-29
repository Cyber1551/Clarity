// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use app::core::commands;
use app::core::state::AppState;

fn main() {
    let state = AppState {
        database_pool: Arc::new(Mutex::new(None))
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_fs_watch::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
          commands::initialize_database,
          commands::get_media_items
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
