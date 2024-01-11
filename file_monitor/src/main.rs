use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{self, BufReader, BufWriter, Read};
use sha2::{Digest, Sha256};
use chrono::{Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use serde_json::{json, Value};


#[derive(Serialize, Deserialize)]
struct Hashes {
    hash: String,
    file_path: String,
    timestamp: String,
}

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
    let empty_array: Vec<Hashes> = Vec::new(); // Create an empty Vec of Hashes
    let json_string = serde_json::to_string(&empty_array)?; // Serialize the empty array to a JSON string
    match fs::write(file_path, json_string) {
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
    let hashes_file = "./data/hashes.json";
    match check_file_exists(hashes_file) {
        Ok(_) => {
            let mut hashes: Vec<Hashes> = match fs::read_to_string(hashes_file) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or(Vec::new()) // Parse the existing content into a Vec<Hashes>
                },
                Err(_) => Vec::new(), // If the file doesn't exist or is empty, create a new Vec<Hashes>
            };

            let new_hash = Hashes {
                hash: hash.to_string(),
                file_path: file_path.to_string(),
                timestamp: creation_timestamp.to_string(),
            };

            hashes.push(new_hash);
            let json_string = serde_json::to_string_pretty(&hashes)?; // Serialize the Vec back to a JSON string
            fs::write(hashes_file, json_string)?; // Write the updated JSON string back to the file

            Ok(String::from("Added to hashes.json"))
        }
        Err(_err) => {
            match create_file(hashes_file) {
                Ok(_) => {
                    let _ = write_hash(hash, file_path, creation_timestamp);
                    Ok(String::from("Ok"))
                },
                Err(err) => Err(err),
            }
        }
    }
}

// Supports Add File. Adds file path to dirs.json
fn add_file(file_path: &str) -> Result<String, io::Error> {
    let dir_path = "./data/dirs.json";

    let mut file = OpenOptions::new().read(true).write(true).open(dir_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse the JSON into a serde_json Value
    let mut data: Value = serde_json::from_str(&contents)?;

    let new_object = json!({"file_path": file_path});

    // Add the new object to the array
    if let Some(array) = data.as_array_mut() {
        array.push(new_object);
    }

    // Write the modified JSON back to the file
    let file = File::create(dir_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &data)?;

    Ok(String::from("Ok"))
}

fn full_scan(file_path: &str) -> Result<String, io::Error> {
    match check_file_exists(file_path) {
        Ok(_) => {
            // Read the JSON file
            let mut file = File::open(file_path).expect("File not found");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Error reading the file");

            // Parse the JSON data
            let json_data: serde_json::Value = serde_json::from_str(&contents).expect("Error parsing JSON");

            if let Some(obj) = json_data.as_array(){
                let _ = fs::remove_file("./data/hashes.json")?; // Delete file before writing new hashes to avoid duplicates
                for i in obj {

                    let line: String = i["file_path"].as_str().unwrap_or("default_path").to_string();
                    let the_path = PathBuf::from(line);

                    if let Ok(entries) = std::fs::read_dir(the_path) { // If the directory is found in the user system
                        for entry in entries {
                            let entry = entry?;
                            let path = entry.path();
                            if path.is_dir() {
                                println!("path is dir");
                            } else {
                                let path = format!("{}", path.to_string_lossy()); // Convert PathBuff to str
                                let hash = hash_file(&path);
                                let hash_str: &str = &hash;
                                let now = Utc::now();
                                let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();
    
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
                    } else {
                        println!("{} was not found in this system", i["path"]);
                    }
                }
            } else {
                println!("The parsed JSON is not an object");
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
        println!("[G] Generate Hash, [A] Add file, [F] Full Scan, [Q] Quit");

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
                            let _ = add_file(file);
                            println!("\n File added! \n");
                        }
                        Err(err) => {
                            eprintln!("Error reading the file: {}", err);
                        }
                    }
                }
                Err(err) => eprintln!("{}", err),
            }

        } else if input == "f" {
            let _ = full_scan("./data/dirs.json");

        } else if input == "q" {
            break

        } else {
            println!("\n Invalid input \n")
        }
    }
}

fn main() {
    cli_menu();
}