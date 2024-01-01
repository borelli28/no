use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;


fn calculate_sha256(file_path: &str) -> Result<String, std::io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashStorage {
    pub hashes: HashMap<String, String>,
    json_file_path: String,
}

impl HashStorage {
    pub fn new(json_file_path: &str) -> io::Result<Self> {
        let storage = HashStorage {
            hashes: HashMap::new(),
            json_file_path: json_file_path.to_string(),
        };

        Ok(storage)
    }
}

fn main() {
    let json_file_path = "./data/hashes.json";
    HashStorage::new(json_file_path).expect("Failed to create HashStorage");
}