use reqwest::blocking::Client;
use serde_json::{Error, Value};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::{copy, BufWriter};
use std::path::Path;
use url::Url;

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
    println!(" -u, --url <link>   Github sub-directory URL");
}

fn print_version() {
    println!("Clone Github Sub-directory (cgs)");
    println!("Version 1.0.0");
    println!("Copyright (c) 2023 Abhishek Kumar licensed under MIT License.");
}

fn clone_subdir(url: &str) {
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

    let list_content_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
        account, repository, subdir_path, branch
    );

    if let Err(e) = list_content(&list_content_url) {
        println!("Failed to list directory content: {}", e);
    }
}

fn list_content(url: &str) -> Result<(), Error> {
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
                
                if let Err(e) = list_content(dir_url) {
                    println!("Failed to list sub-directory content: {}", e);
                }
            } else if file_type == "file" {
                let file_path = element["path"].as_str()
                    .expect("Invalid filepath in the sub-directory");
                
                let file_dir = Path::new(file_path)
                    .parent()
                    .expect("Invalid directory path of the file")
                    .to_str()
                    .expect("Unable to convert directory path to string");
                
                let file_url = element["download_url"].as_str()
                    .expect("Invalid download file URL");
                
                download_file(file_url, file_dir);
            }
        }
    }

    Ok(())
}

fn download_file(url: &str, path: &str) {
    let file_name = url.split("/").last().expect("Invalid download file URL");
    let client = Client::new();
    let response = client.get(url).send().expect("Failed to get response");
    
    fs::create_dir_all(path).expect("Failed to create directory");
    
    let file_path = format!("{}/{}", path, file_name);
    let file = File::create(&file_path).expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    
    copy(&mut response.bytes().expect("Failed to get bytes").as_ref(), &mut writer)
        .expect("Failed to copy data");
    
    writer.flush().expect("Failed to flush writer");
    println!(" {}", file_path);
}

// Define the main function
fn main() {
    // Get the command line arguments as a vector of strings
    let args: Vec<String> = env::args().collect();

    // Check if there are any arguments
    if args.len() > 1 {
        // Get the first argument as the option
        let option = &args[1];

        // Match the option with different cases
        match option.as_str() {
            // If the option is -h or --help, call the print_help function
            "-h" | "--help" => print_help(),
            // If the option is -v or --version, call the print_version function
            "-v" | "--version" => print_version(),
            // If the option is -u or --url, check if there are 1 more argument as string
            "-u" | "--url" => {
                // Check if there are at least 3 arguments in total
                if args.len() >= 3 {
                    let url_str = match env::args().nth(2) {
                        Some(url) => url,
                        None => {
                            println!("Please provide a URL as an argument");
                            return;
                        }
                    };
                    clone_subdir(&url_str);
                } else {
                    println!("Missing numbers for arguments");
                }
            }
            // If the option is anything else, print an error message
            _ => {
                let url_str = match env::args().nth(1) {
                    Some(url) => url,
                    None => {
                        println!("Please provide a URL as an argument");
                        return;
                    }
                };
                clone_subdir(&url_str);
            }
        }
    } else {
        println!("No options or arguments provided");
    }
}
