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

    pub fn start_monitoring<F>(&mut self, interval: Duration, monitor_function: F)
    where
        F: Fn(&mut HashStorage),
    {
        loop {
            monitor_function(self);
            std::thread::sleep(interval);
        }
    }
}

fn monitor_file_system(storage: &mut HashStorage) {
    let directory_to_monitor = ".";
    println!("Directory to monitor: {}", {directory_to_monitor});

    // Get the list of files in the directory
    if let Ok(entries) = fs::read_dir(directory_to_monitor) {
        // Iterate through each element in directory
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

                        // Update the hash map in HashStorage
                        storage
                            .add_hash(&file_path.to_string_lossy())
                            .expect("Failed to add hash");
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

fn main() {
    let json_file_path = "../data/hashes.json";
    let mut storage = HashStorage::new(json_file_path).expect("Failed to create HashStorage");

    storage.start_monitoring(Duration::from_secs(5), monitor_file_system);

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
