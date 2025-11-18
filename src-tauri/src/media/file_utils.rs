use std::ffi::OsStr;
use std::path::Path;

pub fn get_extension(path: &str) -> &str {
    Path::new(path).extension().and_then(OsStr::to_str).unwrap_or("unknown")
}
