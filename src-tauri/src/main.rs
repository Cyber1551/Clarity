// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{http::ResponseBuilder};

mod models;
mod database;
mod media;
mod state;
mod commands;
mod thumbnail;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs_watch::init())
    .register_uri_scheme_protocol("thumbnail", move |_app_handle, request| {
        println!("YOYO {:?}", request.uri());
        print_type_of(&_app_handle);
        print_type_of(&request);
        thumbnail::register_thumbnail_protocol();

        ResponseBuilder::new()
            .mimetype("image/png")
            .status(200)
            .body(Vec::new())
    })
    .invoke_handler(tauri::generate_handler![
      commands::init_database,
      commands::extract_video_metadata,
      commands::extract_image_metadata,
      commands::get_all_media,
      commands::add_tag,
      commands::add_bookmark,
      commands::delete_media_item_by_path,
      commands::update_media_item_path,
      commands::check_thumbnail_exists,
      commands::get_thumbnail,
      commands::generate_thumbnail,
      commands::scan_directory
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
