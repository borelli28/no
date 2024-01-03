use std::collections::HashMap;
use std::fs::File;
use std::fs;
use std::io::{self, BufReader, Read, Write};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;


fn calculate_sha256(file_path: &str) -> Result<String, std::io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hash = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }

    let result = hash.finalize();
    Ok(format!("{:x}", result))
}

// Individual file hash verification
fn hash_file(file_path: &str) -> String {
    let result = calculate_sha256(file_path);

    match result {
        Ok(hash) => {
            return hash
        }
        Err(err) => {
            return err.to_string()
        }
    }
}

// fn monitor(file_path: &str) -> io::Result<()> {
//     let directories_file = file_path;
// }

fn check_file_exists(file_path: &str) -> io::Result<()> {
    if !fs::metadata(file_path).is_ok() {
        fs::write(file_path, "")?;
    }
    Ok(())
}

fn cli_menu() {
    loop {
        println!("[G] Generate Hash, [A] Add file to monitoring list, [Q] Quit");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input: String = input.trim().to_lowercase();
    
        if input == "g" {
            // ./test.txt
            println!("\n Path to file: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            let hash = hash_file(&file);
            println!("\n {} \n", hash);

        } else if input == "q" {
            break

        } else if input == "a" {
            println!("\n Path to file: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            let hash = hash_file(&file);
            println!("\n {} \n", hash);

        } else {
            println!("\n Invalid input \n")
        }
    }
}

fn main() {
    let hashes_db = String::from("./data/hashes.json");
    HashStorage::new(hashes_db).expect("Failed to create HashStorage");

    cli_menu();
}