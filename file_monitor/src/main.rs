use std::fs::File;
use std::io::{self, Read};
use sha2::{Digest, Sha256};

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



fn main() {
    println!("Hello, world!");
}
