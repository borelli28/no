use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{self, BufReader, BufWriter, Read};
use sha2::{Digest, Sha256};
use chrono::{Utc};
use serde::{Serialize, Deserialize};
use std::path::{PathBuf, Path};
use serde_json::{json, Value};
use notify::{Watcher, RecursiveMode, recommended_watcher, Event};
// use notify::Result as NotifyResult;


#[derive(Serialize, Deserialize)]
struct Hashes {
    hash: String,
    file_path: String,
    timestamp: String,
}

fn gen_dirs_file() -> Result<String, io::Error> {
    let file = "./data/dirs.json";

    let data = vec![
        json!({"file_path": "/bin"}),
        json!({"file_path": "/sbin"}),
        json!({"file_path": "/usr/bin"}),
        json!({"file_path": "/usr/sbin"}),
        json!({"file_path": "/usr/local/bin"}),
        json!({"file_path": "/usr/local/sbin"}),
        json!({"file_path": "/lib"}),
        json!({"file_path": "/lib64"}),
        json!({"file_path": "/usr/lib"}),
        json!({"file_path": "/usr/lib64"}),
        json!({"file_path": "/usr/local/lib"}),
        json!({"file_path": "/usr/local/lib64"}),
        json!({"file_path": "/etc"}),
        json!({"file_path": "/boot"}),
        json!({"file_path": "/Library"}),
        json!({"file_path": "/var/log"}),
        json!({"file_path": "/etc/init.d"}),
        json!({"file_path": "/etc/launchd.conf"}),
        json!({"file_path": "/lib/modules"}),
        json!({"file_path": "/System/Library/Extensions"}),
        json!({"file_path": "/etc/cron.d"}),
        json!({"file_path": "/usr/lib/cron/tabs"}),
        json!({"file_path": "/etc/network"}),
        json!({"file_path": "/etc/security"}),
        json!({"file_path": "/etc/ssh"}),
        json!({"file_path": "/var/www"}),
        json!({"file_path": "/Library/WebServer/Documents"}),
        json!({"file_path": "/var/lib/mysql"}),
        json!({"file_path": "/var/lib/postgresql"}),
        json!({"file_path": "/usr/local/var/mysql"}),
        json!({"file_path": "/usr/local/var/postgres"}),
        json!({"file_path": "/System/Library"}),
        json!({"file_path": "/usr/libexec"})
    ];

    let json_data = serde_json::to_string_pretty(&data).unwrap();
    fs::write(file, json_data).unwrap();
    Ok(String::from("Ok"))
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

fn get_hash(file_path: &str) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string("./data/baseline.json")?;

    // Parse the JSON into a serde_json Value
    let data: Value = serde_json::from_str(&contents)?;

    // Search for the object
    if let Some(array) = data.as_array() {
        for obj in array {
            if let Some(path) = obj.get("file_path") {
                if let Some(obj_path) = path.as_str() {
                    if obj_path == file_path {
                        return Ok(obj.to_string());
                    }
                }
            }
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Object not found in baseline.json"))
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
        Err(_) => {
            return Err(io::Error::new(io::ErrorKind::Other, "An error occurred -- check_file_exists"));
        }
    }
}

fn write_hash(hash: &str, file_path: &str, creation_timestamp: &str) -> Result<String, io::Error> {
    let alerts_file = "./data/baseline.json";
    match check_file_exists(alerts_file) {
        Ok(_) => {
            let mut hashes: Vec<Hashes> = match fs::read_to_string(alerts_file) {
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
            fs::write(alerts_file, json_string)?; // Write the updated JSON string back to the file

            Ok(String::from("Added to baseline.json"))
        }
        Err(_err) => {
            match create_file(alerts_file) {
                Ok(_) => {
                    let _ = write_hash(hash, file_path, creation_timestamp);
                    Ok(String::from("Ok"))
                },
                Err(err) => Err(err),
            }
        }
    }
}

fn delete_hash(hash_file_path: &str) -> Result<String, io::Error> {
    let search_for_this_path = hash_file_path;
    let file_path = "./data/baseline.json";

    let contents = fs::read_to_string(file_path)?;

    // Parse the JSON into a serde_json Value
    let mut data: Value = serde_json::from_str(&contents).expect("Error parsing JSON");

    // Search for the object and remove it if found
    if let Some(array) = data.as_array_mut() {
        array.retain(|obj| {
            if let Some(path) = obj.get("file_path") {
                if let Some(path_str) = path.as_str() {
                    return path_str != search_for_this_path;
                }
            }
            true
        });
    }

    // Write the modified JSON back to the file
    let new_contents = serde_json::to_string_pretty(&data)?;
    fs::write(file_path, new_contents)?;

    Ok(String::from("Ok"))
}

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

fn gen_alert(file_path: &str, change_type: &str) -> Result<String, io::Error> {
    let alerts_file = "./data/alerts.json";
    match check_file_exists(alerts_file) {
        Ok(_) => {
            let note: &str = &format!("Change detected in {} since the last scan of the file", file_path);
            let now = chrono::Utc::now();
            let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();
        
            let existing_json = if let Ok(contents) = fs::read_to_string(alerts_file) {
                serde_json::from_str(&contents).unwrap_or(json!({}))
            } else {
                json!({})
            };
        
            let new_alert = json!({
                "file_path": file_path,
                "change_type": change_type,
                "note": note,
                "timestamp": timestamp,
            });
        
            let mut alerts_array = match existing_json {
                Value::Array(arr) => arr,
                _ => vec![existing_json],
            };
        
            alerts_array.push(new_alert);
            let updated_json = serde_json::to_string_pretty(&alerts_array)?;
        
            fs::write(alerts_file, updated_json)?;
        
            Ok(String::from("Ok"))
        }
        Err(_err) => {
            match create_file(alerts_file) {
                Ok(_) => {
                    let _ = gen_alert(file_path, "None");
                    Ok(String::from("Ok"))
                },
                Err(err) => Err(err),
            }
        }
    }
}

fn clear_data() -> Result<String, io::Error> {
    fs::remove_file("./data/dirs.json")?;
    fs::remove_file("./data/baseline.json")?;
    fs::remove_file("./data/alerts.json")?;
    Ok(String::from("Ok"))
}

fn hash_mismatch_checker(hash: &str, file_path: &str) -> bool {
    match get_hash(&file_path) {
        Ok(response) => {
            // Parse the JSON response into a serde_json Value
            let response_json: Value = serde_json::from_str(&response).expect("Error parsing response_str");

            // Access the "hash" field of the JSON object and compare with the provided hash
            if let Some(hash_value) = response_json.get("hash") {
                if let Some(hash_str) = hash_value.as_str() {
                    if hash_str == hash {
                        // println!("{} == {} ?", hash_str, hash);
                        return true;
                    } else if hash_str != hash {
                        return false;
                    } else {
                        return true;
                    }
                } else {
                    return true;
                }
            } else {
                eprintln!("Could not access response_json in hash_mismatch_checker");
                return true;
            }
        }
        Err(_) => {   // Object not found, returned by get_hash()
            return true;
        }
    }
}

fn monitor() -> Result<Event, notify::Error> {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                tx.send(event).unwrap();
            }
        }).unwrap();

        watcher.watch(Path::new("."), RecursiveMode::Recursive).unwrap();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    loop {
        match rx.recv() {
            Ok(event) => {
                match event.kind {
                    notify::EventKind::Create(_) => println!("File created"),
                    notify::EventKind::Modify(_) => println!("File modified"),
                    notify::EventKind::Remove(_) => println!("File removed"),
                    notify::EventKind::Other => println!("Other kind of event"),
                    notify::EventKind::Any => println!("Any"),
                    notify::EventKind::Access(_) => println!("File accessed"),
                }
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
}

fn full_scan(file_path: &str) -> Result<String, io::Error> {
    match check_file_exists(file_path) {
        Ok(_) => {
            println!("Reading directories... Please don't quit the program until it's complete.");

            let mut file = File::open(file_path).expect("File not found");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Error reading the file");

            let json_data: serde_json::Value = serde_json::from_str(&contents).expect("Error parsing JSON");

            if let Some(obj) = json_data.as_array() {
                let obj_length = obj.len();

                if obj_length > 0 {
                    for i in obj {
    
                        let line: String = i["file_path"].as_str().unwrap_or("default_path").to_string();
                        let the_path = PathBuf::from(line);
    
                        if let Ok(entries) = std::fs::read_dir(the_path) { // Return true if directory is traversable, it's found
                            for entry in entries {
                                let entry = entry?;
                                let path = entry.path();
                                if path.is_dir() {
                                    // println!("path is dir");
                                    continue
                                } else {
                                    let path = format!("{}", path.to_string_lossy()); // Convert PathBuff to str
                                    let hash = hash_file(&path);
                                    let hash_str: &str = &hash;
                                    let now = Utc::now();
                                    let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();
                                    
                                    // Check for hash mismatch
                                    if !hash_mismatch_checker(&hash_str, &path) {
                                        let _ = gen_alert(&path, "None");
                                    }

                                    // Delete previous object from file before writing the new object
                                    let _ = delete_hash(&path);
    
                                    match write_hash(hash_str, &path, timestamp) {
                                        Ok(_) => {
                                            // println!("Write Ok");
                                            continue
                                        }
                                        Err(err) => {
                                            eprintln!("Error: {}", err);
                                        }
                                    }
                                }
                            }
                        } else { // String is a file path instead of a directory path
                            // println!("{} path is a file instead of directory, but no biggy...", i["file_path"]);
                            let _line: String = i["file_path"].as_str().unwrap_or("default_path").to_string();
                            let hash = hash_file(&_line);
                            let hash_str: &str = &hash;
                            let now = Utc::now();
                            let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();

                            // Check for hash mismatch
                            if !hash_mismatch_checker(&hash_str, &_line) {
                                let _ = gen_alert(&_line, "None");
                            }
    
                            // Delete previous object from file before writing the new object
                            let _ = delete_hash(&_line);
    
                            match write_hash(hash_str, &_line, timestamp) {
                                Ok(_) => {
                                    // println!("Write Ok");
                                    continue
                                }
                                Err(err) => {
                                    eprintln!("Error: {}", err);
                                }
                            }
                        }
                    }
                } else {
                    println!("File not found");
                }
            } else {
                println!("The parsed JSON is not an object");
            }
            Ok(String::from("Ok"))
        }
        Err(_) => {
            match gen_dirs_file() {
                Ok(_) => {
                    let _ = full_scan(file_path);
                    Ok(String::from("Ok"))
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return Err(io::Error::new(io::ErrorKind::Other, "An error occurred"));
                }
            }
        }
    }
}

fn cli_menu() {
    loop {
        println!("[G] Generate Hash, [A] Add file, [H] Check Hash, [F] Full Scan, [M] Monitor Mode, [C] Clear Data, [Q] Quit");

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

        }  else if input == "h" {
            println!("\n Enter file path: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            let hash = hash_file(file);
            let hash: &str = &hash;

            if !hash_mismatch_checker(hash, file) {
                println!("Hash mismatch found");
            }

        } else if input == "f" {
            let _ = full_scan("./data/dirs.json");

        } else if input == "m" {
            let _ = monitor();

        } else if input == "c" {
            let _ = clear_data();

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