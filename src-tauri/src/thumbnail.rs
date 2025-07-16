use tauri::{http::ResponseBuilder};
use rusqlite::{Connection, params};
use crate::state;


pub fn register_thumbnail_protocol() {
    println!("THIS");
}

// Register the thumbnail URI scheme protocol
// pub fn register_thumbnail_protocol(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
//     app.register_uri_scheme_protocol("thumbnail", move |_app_handle, request| {
//         // Extract the thumbnail ID from the URI
//         if let Some(id_str) = request.uri().strip_prefix("thumbnail://") {
//             // Parse the thumbnail ID
//             match id_str.parse::<i64>() {
//                 Ok(thumbnail_id) => {
//                     // Get the database path
//                     let db_path = match state::get_db_path() {
//                         Ok(path) => path,
//                         Err(_) => return ResponseBuilder::new().status(500).body(Vec::new()),
//                     };
//
//                     // Open a new database connection
//                     let conn = match Connection::open(&db_path) {
//                         Ok(conn) => conn,
//                         Err(_) => return ResponseBuilder::new().status(500).body(Vec::new()),
//                     };
//
//                     // Get the thumbnail directly by its ID
//                     let mut stmt = match conn.prepare(
//                         "SELECT id, media_id, data, mime_type
//                          FROM thumbnails
//                          WHERE id = ?1"
//                     ) {
//                         Ok(stmt) => stmt,
//                         Err(_) => {
//                             return ResponseBuilder::new().status(500).body(Vec::new());
//                         }
//                     };
//
//                     let mut rows = match stmt.query(params![thumbnail_id]) {
//                         Ok(rows) => rows,
//                         Err(_) => {
//                             return ResponseBuilder::new().status(500).body(Vec::new());
//                         }
//                     };
//
//                     match rows.next() {
//                         Ok(Some(row)) => {
//                             // Get the thumbnail data and mime type
//                             let data: Vec<u8> = match row.get(2) {
//                                 Ok(data) => data,
//                                 Err(_) => {
//                                     return ResponseBuilder::new().status(500).body(Vec::new());
//                                 }
//                             };
//
//                             let mime_type: String = match row.get(3) {
//                                 Ok(mime_type) => mime_type,
//                                 Err(_) => {
//                                     return ResponseBuilder::new().status(500).body(Vec::new());
//                                 }
//                             };
//
//                             // Return the thumbnail data
//                             ResponseBuilder::new()
//                                 .mimetype(&mime_type)
//                                 .status(200)
//                                 .body(data)
//                         },
//                         Ok(None) => {
//                             ResponseBuilder::new().status(404).body(Vec::new())
//                         },
//                         Err(_) => {
//                             ResponseBuilder::new().status(500).body(Vec::new())
//                         }
//                     }
//                 },
//                 Err(_) => {
//                     ResponseBuilder::new().status(400).body(Vec::new())
//                 }
//             }
//         } else {
//             ResponseBuilder::new().status(400).body(Vec::new())
//         }
//     });
//
//     Ok(())
// }
