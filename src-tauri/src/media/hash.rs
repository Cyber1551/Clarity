use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use blake3::{Hash, Hasher};

pub fn hash_path(path: &PathBuf) -> Result<Hash, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    Ok(hash_file(file))
}

fn hash_file(file: File) -> Hash {
    let mut reader = BufReader::new(file);
    let mut hasher = Hasher::new();

    let mut buffer = [0u8; 8192]; // 8 KB buffer
    loop {
        let bytes_read = reader.read(&mut buffer).unwrap_or(0);
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    hasher.finalize()
}