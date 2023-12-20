use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use chrono::{Utc};

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

// HashStorage struct and implementation
#[derive(Debug, Serialize, Deserialize)]
pub struct HashStorage {
    pub hashes: HashMap<String, String>,
    json_file_path: String,
}

impl HashStorage {
    pub fn new(json_file_path: &str) -> io::Result<Self> {
        let mut storage = HashStorage {
            hashes: HashMap::new(),
            json_file_path: json_file_path.to_string(),
        };

        storage.load_from_file()?;
        Ok(storage)
    }

    pub fn add_hash(&mut self, file_path: &str) -> io::Result<()> {
        match calculate_sha256(file_path) {
            Ok(hash) => {
                self.hashes.insert(file_path.to_string(), hash);
                self.save_to_file()?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn get_hash(&self, file_path: &str) -> Option<&String> {
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

    pub fn start_monitoring<F>(&mut self, monitor_function: F)
    where
        F: Fn(&mut HashStorage),
    {
        monitor_function(self);
    }    
}

fn monitor_file_system(storage: &mut HashStorage) {
    let directory_to_monitor = ".";
    println!("Directory to monitor: {}", directory_to_monitor);

    // Get the list of files in the directory
    if let Ok(entries) = fs::read_dir(directory_to_monitor) {
        // Iterate through each element in the directory
        for entry in entries {
            if let Ok(entry) = entry {
                println!("entry Ok :)");
                let file_path = entry.path();

                // Check if the entry is a file
                if file_path.is_file() {
                    println!("path is file......");
                    // Calculate the SHA-256 hash for the file
                    if let Ok(hash) = calculate_sha256(&file_path.to_string_lossy()) {
                        println!("File: {:?}, Hash: {}", file_path, hash);

                        // Check if the calculated hash matches the stored hash
                        if let Some(stored_hash) = storage.get_hash(&file_path.to_string_lossy()) {
                            if hash != *stored_hash {
                                println!("Alert: Hash mismatch for {:?}", file_path);

                                // Log the inconsistency to "/data/inconsistencies.json"
                                log_alerts(&file_path.to_string_lossy(), hash, stored_hash);
                            } else {
                                // Update the hash map in HashStorage only if there is no inconsistency
                                storage.add_hash(&file_path.to_string_lossy()).expect("Failed to add hash");
                            }
                        } else {
                            println!("Alert: File not found in hashes.json for {:?}", file_path);
                            storage.add_hash(&file_path.to_string_lossy()).expect("Failed to add hash");
                        }
                    } else {
                        println!("Failed to calculate hash for {:?}", file_path);
                    }
                }
            } else {
                println!("entry not Ok :(");
            }
        }
    }
}

fn log_alerts(file_path: &str, calculated_hash: String, stored_hash: &String) {
    let inconsistency = serde_json::json!({
        "file_path": file_path,
        "calculated_hash": calculated_hash,
        "stored_hash": stored_hash,
        "timestamp": Utc::now().to_rfc3339(),  // Use Utc::now() to get the current time in UTC
        "message": "Hash mismatch",
    });

    let data_dir = Path::new("./data");
    // Ensure the "/data" directory exists, creating it if necessary
    if !data_dir.exists() {
        fs::create_dir(data_dir).expect("Failed to create data directory");
    }

    // Open "/data/alerts.json" file for appending or create if it doesn't exist
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir.join("alerts.json"))
    {
        writeln!(file, "{}", inconsistency.to_string()).expect("Failed to write alerts to file");
    } else {
        println!("Failed to open /data/alerts.json for writing");
    }
}

fn main() {
    let json_file_path = "./data/hashes.json";
    let mut storage = HashStorage::new(json_file_path).expect("Failed to create HashStorage");

    storage.start_monitoring(monitor_file_system);
}
