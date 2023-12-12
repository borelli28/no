
use std::time::{Duration, SystemTime};
use hash_db::hash_storage::HashStorage;
use hash_generator::calculate_sha256;

fn main() {
    // Specify the path to the JSON file for storing the hash map
    let json_file_path = "hashes.json";

    // Create a new instance of HashStorage from the hash_db crate
    let mut storage = HashStorage::new(json_file_path).expect("Failed to create HashStorage");

    // Start monitoring the file system at a specified interval (e.g., every 5 seconds)
    storage.start_monitoring(Duration::from_secs(5), monitor_file_system);

    // Keep the monitoring thread alive
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

// Function to perform the actual monitoring logic
fn monitor_file_system(storage: &mut HashStorage) {
    // Specify the directory to monitor
    let directory_to_monitor = "file/path";

    // Get the list of files in the directory
    if let Ok(entries) = fs::read_dir(directory_to_monitor) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                
                // Check if the entry is a file
                if file_path.is_file() {
                    // Calculate the SHA-256 hash for the file
                    if let Ok(hash) = hash_db::hash_generator::calculate_sha256(&file_path) {
                        println!("File: {:?}, Hash: {}", file_path, hash);

                        // Update the hash map in HashStorage
                        storage.add_hash(&file_path.to_string_lossy()).expect("Failed to add hash");
                    } else {
                        println!("Failed to calculate hash for {:?}", file_path);
                    }
                }
            }
        }
    }

    // Optionally, you can implement logic to remove entries from the hash map
    // that correspond to files that no longer exist in the monitored directory.
}
