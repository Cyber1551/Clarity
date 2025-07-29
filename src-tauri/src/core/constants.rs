// Database
pub const DB_NAME: &str = "media_cache.db";
pub const MAX_POOL_SIZE: u32 = 3;

// Thumbnails
pub const THUMBNAIL_SIZE: u32 = 256;
pub const THUMBNAIL_EXTENSION: &str = "webp";
pub const BROKEN_THUMBNAIL: &[u8] = include_bytes!("../../assets/broken_thumbnail.webp");
