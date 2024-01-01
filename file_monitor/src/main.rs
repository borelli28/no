use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;


fn calculate_sha256(file_path: &str) -> Result<String, std::io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hash = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }

    let result = hash.finalize();
    Ok(format!("{:x}", result))
}

#[derive(Serialize, Deserialize)]
struct HashStorage {
    hashes: HashMap<String, String>,
    json_file_path: String,
}

impl HashStorage {
    fn new(json_file_path: &str) -> io::Result<Self> {
        let storage = HashStorage {
            hashes: HashMap::new(),
            json_file_path: json_file_path.to_string(),
        };

        Ok(storage)
    }

    fn save_to_file(&self) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(&self.hashes)?;

        let mut file = File::create(&self.json_file_path)?;
        file.write_all(json_data.as_bytes())?;

        Ok(())
    }

    fn add_hash(&mut self, file_path: &str) -> io::Result<()> {
        match calculate_sha256(file_path) {
            Ok(hash) => {
                self.hashes.insert(file_path.to_string(), hash);
                self.save_to_file()?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

fn main() {
    let json_file_path = "./data/hashes.json";
    HashStorage::new(json_file_path).expect("Failed to create HashStorage");
}