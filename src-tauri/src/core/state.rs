use std::path::PathBuf;
use std::sync::Mutex;

pub struct LibraryRootState(pub Mutex<Option<PathBuf>>);
