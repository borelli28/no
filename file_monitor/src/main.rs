use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{self, BufReader, Read, Write};
use sha2::{Digest, Sha256};
use chrono::{Utc, Datelike, Timelike};
use serde_json::json;


fn calculate_sha256(file_path: &str) -> Result<String, io::Error> {
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

fn check_file_exists(file_path: &str) -> Result<String, io::Error> {
    if !fs::metadata(file_path).is_ok() {
        fs::write(file_path, "")?;
    }
    Ok(String::from("Ok"))
}

fn write_hash(hash: &str, file_path: &str, creation_timestamp: &str) -> Result<String, io::Error> {
    match check_file_exists(file_path) {
        Ok(response) => {
            let mut file = OpenOptions::new().append(true).open(file_path)?;

            let text = json!({
                "hash": hash,
                "file_path": file_path,
                "creation_timestamp": creation_timestamp
            }).to_string();

            file.write_all(b"\n")?;
            file.write_all(text.as_bytes())?;

            Ok(String::from("Ok"))
        }
        Err(err) => Err(err),
    }
}

fn cli_menu() {
    let hashes_json = "./data/hashes.json";

    loop {
        println!("[G] Generate Hash, [A] Add file, [Q] Quit");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input: String = input.trim().to_lowercase();
    
        if input == "g" {
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
            let hash = hash.as_str();
            let now = Utc::now();
            let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();

            write_hash(hash, hashes_json, timestamp);

        } else {
            println!("\n Invalid input \n")
        }
    }
}

fn main() {
    cli_menu();
}