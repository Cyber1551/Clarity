// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{http::ResponseBuilder};

mod models;
mod database;
mod types;
mod media;
mod state;
mod commands;
mod thumbnail;
mod media_helpers;
mod image_helpers;
mod video_helpers;
mod constants;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs_watch::init())
    .invoke_handler(tauri::generate_handler![
      commands::initialize_database,
      commands::extract_video_metadata,
      commands::extract_image_metadata,
      commands::get_all_media,
      commands::delete_media_item_by_path,
      commands::update_media_item_path,
      commands::check_thumbnail_exists,
      commands::get_thumbnail,
      commands::generate_thumbnail,
      commands::scan_directory,
      commands::update_media_cache
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
