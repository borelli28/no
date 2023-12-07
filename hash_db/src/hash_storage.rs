use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use hash_generator::calculate_sha256;


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

    pub fn get_hash(&self, file_path: &str) -> Option<&String> {
        self.hashes.get(file_path)
    }

    // Load existing hashes from the JSON file
    fn load_from_file(&mut self) -> io::Result<()> {
        // Check if the JSON file exists
        if Path::new(&self.json_file_path).exists() {
            // Read the JSON data from the file
            let json_data = fs::read_to_string(&self.json_file_path)?;
            // Deserialize the JSON data into the hash map
            self.hashes = serde_json::from_str(&json_data)?;
        }
        Ok(())
    }

    // Save the hash map to the JSON file
    fn save_to_file(&self) -> io::Result<()> {
        // Serialize the hash map to JSON format
        let json_data = serde_json::to_string_pretty(&self.hashes)?;

        // Create or open the JSON file
        let mut file = File::create(&self.json_file_path)?;
        file.write_all(json_data.as_bytes())?;

        Ok(())
    }
}
