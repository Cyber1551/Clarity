pub const CONFIG_FILE_NAME: &str = "config.json";
pub const FFMPEG_BIN: &str = "bin/ffmpeg";
pub const FFPROBE_BIN: &str = "bin/ffprobe";
pub const WORKER_THREAD_SLEEP_DURATION: std::time::Duration = std::time::Duration::from_millis(1000);

// Database
pub const DB_NAME: &str = ".media-filesystem.sqlite3";

// Thumbnails
pub const THUMBNAIL_SIZE: u32 = 256;
pub const THUMBNAIL_EXTENSION: &str = "webp";
pub const BROKEN_THUMBNAIL: &[u8] = include_bytes!("../../assets/broken_thumbnail.webp");

// Directories
pub const OBJECTS_DIRECTORY: &str = ".objects";
pub const UNSORTED_DIRECTORY: &str = "Unsorted Media";
pub const SORTED_DIRECTORY: &str = "Sorted Media";

// Media Types
pub const VALID_IMAGE_EXTENSIONS: [&str; 8] = ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "svg", "webp"];
pub const VALID_VIDEO_EXTENSIONS: [&str; 7] = ["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"];