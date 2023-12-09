use std::time::Duration;
use hash_db::hash_storage::HashStorage;

fn main() {
    // Specify the path to the JSON file for storing the hash map
    let json_file_path = "hashes.json";

    // Create a new instance of HashStorage from the hash_db crate
    let mut storage = HashStorage::new(json_file_path).expect("Failed to create HashStorage");

    // Start monitoring the file system at a specified interval (e.g., every 5 seconds)
    storage.start_monitoring(Duration::from_secs(5));

    // Keep the monitoring thread alive
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
