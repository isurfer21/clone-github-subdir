use reqwest::blocking::Client;
use serde_json::{Error, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::{copy, BufWriter};
use std::path::{Path, PathBuf};
use std::process;
use url::Url;
use regex::Regex;
use base64::{Engine as _, engine::general_purpose};

fn print_help() {
    println!("Usage:");
    println!(" cgs [options] <link>");
    println!("");
    println!("Arguments:");
    println!(" link              Github sub-directory URL");
    println!("");
    println!("Options:");
    println!(" -h, --help         Show this help message");
    println!(" -v, --version      Show the program version");
    println!(" -c, --curdir       Current sub-directory only");
}

fn print_version() {
    println!("Clone Github Sub-directory (cgs)");
    println!("Version 1.0.0");
    println!("Copyright (c) 2023 Abhishek Kumar licensed under MIT License.");
}

fn parse_args() -> (HashMap<String, String>, Vec<String>) {
    let mut flags = HashMap::new();
    let mut args = Vec::new();
    let flag_with_value = Regex::new(r"^[\-\-]{1,2}.+[\=\:].*$").unwrap();
    let flag_without_value = Regex::new(r"^[\-\-]{1,2}.+$").unwrap();

    for arg in env::args() {
        match arg.as_str() {
            s if flag_with_value.is_match(s) => {
                let parts = s.splitn(2, |c| c == '=' || c == ':').collect::<Vec<&str>>();
                let key = parts[0].trim_start_matches('-');
                let value = parts[1];
                flags.insert(key.to_string(), value.to_string());
            }
            s if flag_without_value.is_match(s) => {
                let key = s.trim_start_matches('-');
                flags.insert(key.to_string(), true.to_string());
            }
            s => {
                args.push(s.to_string());
            }
        }
    }

    (flags, args)
}

fn strip_dir_path(dir_path: &str, dir_name: &str) -> String {
    let path = Path::new(dir_path);
    let path_segments = path.iter();

    let mut stripped_path_segments = PathBuf::new();
    let mut found = false;

    for segment in path_segments {
        if segment == dir_name {
            found = true;
        }
        if found {
            stripped_path_segments.push(segment);
        }
    }

    let stripped_path: PathBuf = stripped_path_segments.iter().collect();
    let stripped_path_string = stripped_path.to_string_lossy();

    stripped_path_string.into_owned()
}

fn clone_subdir(url: &str, opt: &mut HashMap<&str, bool>) {
    let url_obj = Url::parse(&url)
        .expect("Invalid URL");
    let path_segments = url_obj.path_segments()
        .expect("Cannot split the URL path");
    let account = path_segments.clone().nth(0)
        .expect("No account in the URL path");
    let repository = path_segments.clone().nth(1)
        .expect("No repository in the URL path");
    let _tree = path_segments.clone().nth(2)
        .expect("No tree in the URL path");
    let branch = path_segments.clone().nth(3)
        .expect("No branch in the URL path");
    
    let subdir_path = path_segments.clone().skip(4)
        .collect::<Vec<&str>>().join("/");

    let subdir_name = path_segments.clone().last()
        .expect("No branch in the URL path");
    
    if Path::new(&subdir_path).is_dir() {
        fs::remove_dir_all(subdir_path.clone()).expect("Failed to delete existing directory");
    }

    let list_content_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
        account, repository, subdir_path, branch
    );

    if let Err(e) = list_content(&list_content_url, &subdir_name, opt) {
        println!("Failed to list directory content: {}", e);
    }
}

fn list_content(url: &str, dir: &str, opt: &mut HashMap<&str, bool>) -> Result<(), Error> {
    let client = Client::new();

    let mut response = client.get(url).header("User-Agent", " ").send()
        .expect("Failed to get response from the API");
    
    let mut body = String::new();
    response.read_to_string(&mut body)
        .expect("Failed to read response of the API");
    
    let item_list: Value = serde_json::from_str(&body)?;
    if let Some(array) = item_list.as_array() {
        for element in array {
            let file_type = element["type"].as_str()
                .expect("No file type in the response items");
            if file_type == "dir" {
                let dir_url = element["url"].as_str()
                    .expect("Invalid sub-directory URL");
                
                if let Err(e) = list_content(dir_url, &dir, opt) {
                    println!("Failed to list sub-directory content: {}", e);
                }
            } else if file_type == "file" {
                let file_path = element["path"].as_str()
                    .expect("Invalid filepath in the sub-directory");
                
                if opt["curdir"] {
                    let file_dir_path = Path::new(file_path)
                        .parent()
                        .as_ref()
                        .and_then(|p| p.to_str())
                        .unwrap_or("Invalid directory path of the sub-file");

                    let file_dir = &strip_dir_path(file_dir_path, dir);
                    
                    let file_url = element["download_url"].as_str()
                        .expect("Invalid download file URL");
                    
                    download_file(file_url, file_dir);
                } else {
                    let file_dir = Path::new(file_path)
                        .parent()
                        .as_ref()
                        .and_then(|p| p.to_str())
                        .unwrap_or("Invalid directory path of the file");
                    
                    let file_url = element["download_url"].as_str()
                        .expect("Invalid download file URL");
                    
                    download_file(file_url, file_dir);
                }
            }
        }
    } else if let Some(object) = item_list.as_object() {        
        let file_type = object["type"].as_str()
            .expect("No file type in the response");
        if file_type == "file"{
            let file_encoding = object["encoding"].as_str()
                .expect("No file encoding scheme provided in response");
            if file_encoding == "base64" {
                let file_name = object["name"].as_str().unwrap();
                let file_content = object["content"].as_str().unwrap().replace("\n", "");
                let bytes = &general_purpose::STANDARD.decode(file_content).unwrap();
                fs::write(file_name, &bytes).unwrap();
                println!(" {}", file_name);
            }
        }
    } else {
        println!("Invalid response from GitHub API");
    }

    Ok(())
}

fn download_file(url: &str, path: &str) {
    let file_name = url.split("/").last().expect("Invalid download file URL");
    let client = Client::new();
    let response = client.get(url).send().expect("Failed to get response");

    fs::create_dir_all(path).expect("Failed to create directory");
    
    let file_path: PathBuf = Path::new(path).join(file_name).iter().collect();
    let file = File::create(file_path.clone()).expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    
    copy(&mut response.bytes().expect("Failed to get bytes").as_ref(), &mut writer)
        .expect("Failed to copy data");
    
    writer.flush().expect("Failed to flush writer");
    println!(" {}", file_path.display());
}

fn main() {
    let (flags, args) = parse_args();

    if flags.contains_key("help") || flags.contains_key("h") {
        print_help();
        process::exit(0);
    }

    if flags.contains_key("version") || flags.contains_key("v") {
        print_version();
        process::exit(0);
    }

    let mut options = HashMap::new();

    if flags.contains_key("curdir") || flags.contains_key("c") {
        options.insert("curdir", true);
    } else {
        options.insert("curdir", false);
    }

    if args.len() > 1 {
        let url_str = &args[1];
        clone_subdir(&url_str, &mut options);
    } else {
        println!("No options or arguments provided");
        process::exit(0);
    }
}
