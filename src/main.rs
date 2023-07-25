use std::env; // To access the command line arguments
use std::fs; // To create a directory
use url::Url; // To parse the URL
use std::io::Write; // To write the file
use std::io::Read; // To read from the URL
use std::fs::File; // To create a file
use std::path::Path; // To work with paths
use std::io::{copy, BufWriter}; // To copy data from the URL to the file
use reqwest::blocking::Client; // To make HTTP requests
use serde_json::{Value, Error}; // To parse JSON

fn main() {
    // Get the first command line argument as a string
    let url_str = env::args().nth(1).expect("Please provide a URL as an argument");

    // Parse the URL using the url crate
    let url = Url::parse(&url_str).expect("Invalid URL");

    // Print the various chunks of the URL
    // println!("Scheme: {}", url.scheme());
    // println!("Username: {}", url.username());
    // println!("Password: {:?}", url.password());
    // println!("Host: {:?}", url.host());
    // println!("Port: {:?}", url.port());
    // println!("Path: {}", url.path());
    // println!("Query: {:?}", url.query());
    // println!("Fragment: {:?}", url.fragment());

    // Get the URL path as a vector of strings
    let path_segments = url.path_segments().expect("Cannot split the path");

    // Get the first and second chunks of the path as strings
    let account = path_segments.clone().nth(0).expect("No account in the path");
    let repository = path_segments.clone().nth(1).expect("No repository in the path");
    let _tree = path_segments.clone().nth(2).expect("No tree in the path");
    let branch = path_segments.clone().nth(3).expect("No branch in the path");
    let subdir_path = path_segments.clone().skip(4).collect::<Vec<&str>>().join("/");

    // Print the account and repository variables
    // println!("Account: {}", account);
    // println!("Repository: {}", repository);
    // println!("Tree: {}", tree);
    // println!("Branch: {}", branch);
    // println!("Subdir path: {}", subdir_path);

    let list_content_url = format!("https://api.github.com/repos/{}/{}/contents/{}?ref={}", account, repository, subdir_path, branch);

    // println!("Dir URL: {}", list_content_url);

    let _ = list_content(&list_content_url);
}

fn list_content(url: &str) -> Result<(), Error> {
    // Create a HTTP client
    let client = Client::new();

    // Make a GET request to the URL and get the response
    let mut response = client.get(url).header("User-Agent", " ").send().expect("Failed to get response");

    // Create an empty string to store the response body
    let mut body = String::new();

    // Read the response body into the string
    response.read_to_string(&mut body).expect("Failed to read response");

    // Parse the string as a JSON value
    let item_list: Value = serde_json::from_str(&body)?;

    // Check if the JSON value is an array
    if let Some(array) = item_list.as_array() {
        for element in array {
            let file_type = element["type"].as_str().expect("No file type");
            if file_type == "dir" {
                // println!("{}", element["url"]);
                let dir_url = element["url"].as_str().expect("Invalid url");
                let _ = list_content(dir_url);
            } else if file_type == "file" {
                // println!("{}", element["download_url"]);
                let file_path = element["path"].as_str().expect("Invalid path");
                let file_dir = Path::new(file_path).parent().expect("Invalid directory").to_str().expect("Invalid directory path");
                let file_url = element["download_url"].as_str().expect("Invalid url");
                download_file(file_url, file_dir);
            }
        }
    }

    Ok(())
}

// Define a function to download a file from a given url to a specific path from the current working directory
// The function will create the directory if it doesn't exist
fn download_file(url: &str, path: &str) {
    // Define the file name to save as by splitting the url by "/" and taking the last part
    let file_name = url.split("/").last().expect("Invalid url");

    // Create a HTTP client
    let client = Client::new();

    // Make a GET request to the URL and get the response
    let response = client.get(url).send().expect("Failed to get response");

    // Create the directory with the given path if it doesn't exist
    fs::create_dir_all(path).expect("Failed to create directory");

    // Join the path and the file name to form the full file path
    let file_path = format!("{}/{}", path, file_name);

    // Create a file with the given file path
    let file = File::create(&file_path).expect("Failed to create file");

    // Create a buffered writer for the file
    let mut writer = BufWriter::new(file);

    // Copy the data from the response to the writer
    copy(&mut response.bytes().expect("Failed to get bytes").as_ref(), &mut writer).expect("Failed to copy data");

    // Flush the writer
    writer.flush().expect("Failed to flush writer");

    println!(" {}", file_path);
}
