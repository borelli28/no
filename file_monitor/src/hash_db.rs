mod hash_storage;

use hash_storage::HashStorage;

fn main() {
    // Specify the path to the JSON file for storing the hash map
    let json_file_path = "../data/hashes.json";

    // Create a new instance of HashStorage
    let mut storage = HashStorage::new(json_file_path).expect("Failed to create HashStorage");

    // Add hashes to the storage
    storage
        .add_hash("/path/to/file1.txt")
        .expect("Failed to add hash");

    // Retrieve hash using the file path
    let file_path_to_check = "/path/to/file1.txt";
    match storage.get_hash(file_path_to_check) {
        Some(hash) => println!("{}", hash),
        None => println!("Hash not found for {}", file_path_to_check),
    }
}
