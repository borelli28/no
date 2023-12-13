use std::fs::File;
use std::io::{self, Read};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

pub fn calculate_sha256(file_path: &str) -> Result<String, io::Error> {
    // Open the file
    let mut file = File::open(file_path)?;

    // Create a SHA-256 hasher
    let mut sha256 = Sha256::new();

    // Buffer to read the file in chunks
    let mut buffer = [0; 1024];

    // Loop to read the file in chunks and update the hasher
    loop {
        let bytes_read = file.read(&mut buffer)?;

        // Break the loop when no more bytes can be read
        if bytes_read == 0 {
            break;
        }

        // Update the hasher with the read chunk
        sha256.update(&buffer[..bytes_read]);
    }

    // Finalize the hash and convert it to a hexadecimal string
    let result = sha256.finalize();
    Ok(format!("{:x}", result))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashStorage {
    pub hashes: HashMap<String, String>, // File path -> SHA-256 hash
    json_file_path: String, // Path to the JSON file for storing the hash map
}

impl HashStorage {
    pub fn new(json_file_path: &str) -> io::Result<Self> {
        // Create a new instance of HashStorage
        let mut storage = HashStorage {
            hashes: HashMap::new(),
            json_file_path: json_file_path.to_string(),
        };

        // Load existing hashes from the JSON file, if any
        storage.load_from_file()?;

        Ok(storage)
    }

    pub fn add_hash(&mut self, file_path: &str) -> io::Result<()> {
        // Calculate SHA-256 hash for the given file path
        match calculate_sha256(file_path) {
            Ok(hash) => {
                // Insert the file path and hash into the hash map
                self.hashes.insert(file_path.to_string(), hash);
                // Save the updated hash map to the JSON file
                self.save_to_file()?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}



fn main() {
    println!("Hello, world!");
}
