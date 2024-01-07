use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{self, BufReader, BufRead, Read, Write};
use sha2::{Digest, Sha256};
use chrono::{Utc};
use serde_json::json;
use std::path::{Path, PathBuf};


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

fn create_file(file_path: &str) -> Result<String, io::Error> {
    match fs::write(file_path, "") {
        Ok(_) => {
            Ok(String::from("Ok"))},
        Err(err) => {
            Err(err)
        }
    }
}

fn check_file_exists(file_path: &str) -> Result<String, io::Error> {
    match fs::metadata(file_path) {
        Ok(_) => {
            Ok(String::from("Ok"))},
        Err(err) => {
            Err(err)
        }
    }
}

fn write_hash(hash: &str, file_path: &str, creation_timestamp: &str) -> Result<String, io::Error> {
    match check_file_exists("./data/hashes.json") {
        Ok(_) => {
            let mut file = OpenOptions::new().append(true).open("./data/hashes.json")?;

            let text = json!({
                "hash": hash,
                "file_path": file_path,
                "creation_timestamp": creation_timestamp
            }).to_string();

            file.write_all(b"\n")?;
            file.write_all(text.as_bytes())?;

            Ok(String::from("Added to hashes.json"))
        }
        Err(_err) => {
            match create_file("./data/hashes.json") {
                Ok(_) => {
                    write_hash(hash, file_path, creation_timestamp);
                    Ok(String::from("Ok"))
                },
                Err(err) => Err(err),
            }
        }
    }
}

fn monitor_mode(file_path: &str) -> Result<String, io::Error> {
    match check_file_exists(file_path) {
        Ok(_) => {
            let file = File::open(file_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;

                for entry in fs::read_dir(line)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        println!("path is dir");
                    } else {
                        let path = format!("{}", path.to_string_lossy());
                        let hash = hash_file(&path);
                        let hash_str: &str = &hash;
                        let now = Utc::now();
                        let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();
                        // If the line exists in hashes.json, delete it, then call write_hash()
                        // else, call write_hash()
                        match write_hash(hash_str, &path, timestamp) {
                            Ok(_) => {
                                println!("Ok");
                            }
                            Err(err) => {
                                eprintln!("Error: {}", err);
                            }
                        }
                    }
                }
            }
            Ok(String::from("Ok"))
        }
        Err(err) => {
            Err(err)
            // No file found.
        }
    }
}

fn cli_menu() {
    loop {
        println!("[G] Generate Hash, [A] Add file, [M] Monitor, [Q] Quit");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input: String = input.trim().to_lowercase();
    
        if input == "g" {
            println!("\n Enter file path: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            let response = hash_file(&file);
            println!("\n {} \n", response);

        } else if input == "q" {
            break

        } else if input == "a" {
            println!("\n Enter file path: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            match check_file_exists(file) {
                Ok(_) => {
                    let hash = hash_file(&file);
                    let hash = hash.as_str();
                    let now = Utc::now();
                    let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();
        
                    match write_hash(hash, file, timestamp) {
                        Ok(response) => {
                            println!("\n {} \n", response);
                        }
                        Err(err) => {
                            eprintln!("Error reading the file: {}", err);
                        }
                    }
                }
                Err(err) => eprintln!("{}", err),
            }
        } else if input == "m" {
            println!("Placeholder");
            monitor_mode("./data/unix-dirs.txt");

        } else {
            println!("\n Invalid input \n")
        }
    }
}

fn main() {
    cli_menu();
}