mod hash_generator;

use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{self, BufWriter, Read};
use chrono::{Utc};
use serde::{Serialize, Deserialize};
use std::path::{PathBuf, Path};
use std::collections::HashSet;
use serde_json::{json, Value};
use notify::{Watcher, RecursiveMode, recommended_watcher, Event, ErrorKind};

use crate::hash_generator::hash_file;

#[derive(Serialize, Deserialize)]
struct Hashes {
    hash: String,
    file_path: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct Alert {
    file_path: String,
    event_type: EventType,
    note: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
enum EventType {
    Create,
    Modify,
    Remove,
    Access
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

    let json_data = serde_json::to_string_pretty(&data).expect("Could not convert data to json");

    if let Err(_) = fs::write(file, json_data) {
        fs::create_dir_all("./data").expect("Could not create /data directory");
        let json_data = serde_json::to_string_pretty(&data).expect("Could not convert data to json");
        fs::write(file, json_data).expect("Could not write file");
    }

    Ok(String::from("Ok"))
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

fn gen_alert(file_path: &str, event_type: EventType) -> Result<String, io::Error> {
    let alerts_file = "./data/alerts.json";
    match check_file_exists(alerts_file) {
        Ok(_) => {
            let note: String = format!("Event detected in {}", file_path);
            let now = chrono::Utc::now();
            let timestamp: String = now.format("%Y-%m-%d %H:%M:%S").to_string();
        
            let existing_json = if let Ok(contents) = fs::read_to_string(alerts_file) {
                serde_json::from_str(&contents).unwrap_or(json!({}))
            } else {
                json!({})
            };
        
            let new_alert = Alert {
                file_path: file_path.to_string(),
                event_type: event_type,
                note,
                timestamp,
            };

            let mut alerts_array = match existing_json {
                Value::Array(arr) => arr,
                _ => vec![existing_json],
            };
        
            alerts_array.push(serde_json::to_value(&new_alert)?); // Convert new_alert to Value before pushing into alerts_array
            let updated_json = serde_json::to_string_pretty(&alerts_array)?;
        
            fs::write(alerts_file, updated_json)?;
        
            Ok(String::from("Ok"))
        }
        Err(_err) => {
            match create_file(alerts_file) {
                Ok(_) => {
                    let _ = gen_alert(file_path, event_type);
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
    let mut files_to_watch: HashSet<String> = HashSet::new();

    let mut file = OpenOptions::new().read(true).open("./data/baseline.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: Result<Value, _> = serde_json::from_str(&contents);
    match data {
        Ok(value) => {
            if let Some(arr) = value.as_array() {
                for val in arr {
                    if let Some(file_path) = val.get("file_path").and_then(Value::as_str) {
                        files_to_watch.insert(file_path.to_string());
                    }
                }
            }
        },
        Err(_) => {
            return Err(notify::Error::new(ErrorKind::Generic(String::from("Error parsing data in fn monitor"))));
        }
    }

    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                sender.send(event).unwrap();
            }
        }).unwrap();

        for dir in &files_to_watch {
            if let Ok(_) = fs::metadata(dir) { // fs::metadata is used to check if the path exists to prevent errors
                watcher.watch(Path::new(dir), RecursiveMode::NonRecursive).unwrap();
            }
        }

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    loop {
        match receiver.recv() {
            Ok(event) => {
                let path = event.paths[0].to_str().unwrap_or("None");
    
                match event.kind {
                    notify::EventKind::Create(_) => {
                        let _ = gen_alert(path, EventType::Create);
                    }
                    notify::EventKind::Modify(_) => {
                        let _ = gen_alert(path, EventType::Modify);
                    }
                    notify::EventKind::Remove(_) => {
                        let _ = gen_alert(path, EventType::Remove);
                    }
                    notify::EventKind::Access(_) => {
                        let _ = gen_alert(path, EventType::Access);
                    }
                    notify::EventKind::Other | notify::EventKind::Any => println!("Other kind of event \n"),
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}

fn gen_baseline(file_path: &str) -> Result<String, io::Error> {
    match check_file_exists(file_path) {
        Ok(_) => {
            println!("\nReading directories... Please don't quit the program until it's complete.\n");

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

                        if the_path.exists(){
                            if let Ok(entries) = std::fs::read_dir(the_path) { // Return true if traversable directory is found
                                for entry in entries {
                                    let entry = entry?;
                                    let path = entry.path();
                                    if path.is_dir() {
                                        continue
                                    } else {
                                        let path = format!("{}", path.to_string_lossy()); // Convert PathBuff to str
                                        let hash = hash_file(&path);
                                        let hash_str: &str = &hash;
                                        let now = Utc::now();
                                        let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();

                                        if !hash_mismatch_checker(&hash_str, &path) {
                                            let _ = gen_alert(&path, EventType::Modify);
                                        }

                                        let _ = delete_hash(&path); // Delete previous object from file before writing the new object
        
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
                                let _line: String = i["file_path"].as_str().unwrap_or("default_path").to_string();
                                let hash = hash_file(&_line);
                                let hash_str: &str = &hash;
                                let now = Utc::now();
                                let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();

                                if !hash_mismatch_checker(&hash_str, &_line) {
                                    let _ = gen_alert(&_line, EventType::Modify);
                                }
        
                                let _ = delete_hash(&_line); // Delete previous object from file before writing the new object
        
                                match write_hash(hash_str, &_line, timestamp) {
                                    Ok(_) => {
                                        continue
                                    }
                                    Err(err) => {
                                        eprintln!("Error: {}", err);
                                    }
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
                    let _ = gen_baseline(file_path);
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

fn show_alerts() -> Result<String, io::Error> {
    let mut file = OpenOptions::new().read(true).open("./data/alerts.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: Result<Value, _> = serde_json::from_str(&contents);
    match data {
        Ok(value) => {
            if let Some(arr) = value.as_array() {
                let mut counter: u16 = 1;
                for val in arr {
                    let log = format!("#{} --- {} event in: {}, at: {}", counter, val["event_type"], val["file_path"], val["timestamp"]);
                    println!("{}", log);
                    counter += 1;
                }
                println!("\n");
            }
            Ok(String::from("Ok"))
        },
        Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err.to_string())),
    }
}

fn cli_menu() {
    loop {
        println!("\n[G] Generate Hash, [H] Check File, [A] Add File to Monitor List, [B] Generate Baseline, [M] Monitor Mode, [S] Show Alerts, [C] Clear Data, [Q] Quit\n");

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

        }  else if input == "h" {
            println!("\n Enter file path: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            let hash = hash_file(file);
            let hash: &str = &hash;

            if !hash_mismatch_checker(hash, file) {
                println!("Mismatch found \n");
            } else {
                println!("No mistmatch detected or file not found in baseline \n");
            }

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
                            eprintln!("Error reading the file: {} \n", err);
                        }
                    }
                }
                Err(err) => eprintln!("{}", err),
            }

        } else if input == "b" {
            let _ = gen_baseline("./data/dirs.json");

        } else if input == "m" {
            let _ = monitor();

        } else if input == "s" {
            let _ = show_alerts();

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