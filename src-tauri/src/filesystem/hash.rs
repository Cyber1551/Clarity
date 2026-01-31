use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use crate::core::constants::HASH_BUFFER_SIZE;
use crate::core::error::AppResult;

pub fn compute_hash(path: &Path) -> AppResult<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; HASH_BUFFER_SIZE];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}
