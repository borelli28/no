use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Function to calculate SHA-256 hash for a file
fn calculate_sha256(file_path: &str) -> Result<String, io::Error> {
    let mut file = File::open(file_path)?;

    let mut sha256 = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        sha256.update(&buffer[..bytes_read]);
    }

    let result = sha256.finalize();
    Ok(format!("{:x}", result))
}

#[derive(Debug, Serialize, Deserialize)]
struct HashStorage {
    hashes: HashMap<String, String>,
    json_file_path: String,
}

impl HashStorage {
    fn new(json_file_path: &str) -> io::Result<Self> {
        let mut storage = HashStorage {
            hashes: HashMap::new(),
            json_file_path: json_file_path.to_string(),
        };

        storage.load_from_file()?;
        Ok(storage)
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

    fn get_hash(&self, file_path: &str) -> Option<&String> {
        self.hashes.get(file_path)
    }

    fn load_from_file(&mut self) -> io::Result<()> {
        if Path::new(&self.json_file_path).exists() {
            let json_data = fs::read_to_string(&self.json_file_path)?;
            self.hashes = serde_json::from_str(&json_data)?;
        }
        Ok(())
    }

    fn save_to_file(&self) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(&self.hashes)?;
        let mut file = File::create(&self.json_file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }
}

// Function to perform monitoring logic
fn monitor_file_system(storage: &mut HashStorage) {
    let directory_to_monitor = "file/path";

    if let Ok(entries) = fs::read_dir(directory_to_monitor) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();

                if file_path.is_file() {
                    if let Ok(hash) = calculate_sha256(&file_path.to_string_lossy()) {
                        println!("File: {:?}, Hash: {}", file_path, hash);
                        storage.add_hash(&file_path.to_string_lossy()).expect("Failed to add hash");
                    } else {
                        println!("Failed to calculate hash for {:?}", file_path);
                    }
                }
            }
        }
    }
}

fn main() {
    // Specify the path to the JSON file for storing the hash map
    let json_file_path = "../data/hashes.json";

    // Create a new instance of HashStorage
    let mut storage = HashStorage::new(json_file_path).expect("Failed to create HashStorage");

    // Start monitoring the file system at a specified interval (e.g., every 5 seconds)
    storage.start_monitoring(Duration::from_secs(5), monitor_file_system);

    // Keep the monitoring thread alive
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
