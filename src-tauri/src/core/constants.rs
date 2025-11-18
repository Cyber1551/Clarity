pub const CONFIG_FILE_NAME: &str = "config.json";

// Database
pub const DB_NAME: &str = ".media-cache.sqlite3";

// Thumbnails
pub const THUMBNAIL_SIZE: u32 = 256;
pub const THUMBNAIL_EXTENSION: &str = "webp";
pub const BROKEN_THUMBNAIL: &[u8] = include_bytes!("../../assets/broken_thumbnail.webp");

// Directories
pub const OBJECTS_DIRECTORY: &str = ".objects";
pub const UNSORTED_DIRECTORY: &str = "Unsorted Media";
pub const SORTED_DIRECTORY: &str = "Sorted Media";