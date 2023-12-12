use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

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
   
