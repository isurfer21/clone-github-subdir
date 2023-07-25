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

fn main() {
    let url_str = env::args()
        .nth(1)
        .expect("Please provide a URL as an argument");
    let url = Url::parse(&url_str).expect("Invalid URL");
    let path_segments = url.path_segments().expect("Cannot split the URL path");
    let account = path_segments
        .clone()
        .nth(0)
        .expect("No account in the URL path");
    let repository = path_segments
        .clone()
        .nth(1)
        .expect("No repository in the URL path");
    let _tree = path_segments.clone().nth(2).expect("No tree in the URL path");
    let branch = path_segments.clone().nth(3).expect("No branch in the URL path");
    let subdir_path = path_segments
        .clone()
        .skip(4)
        .collect::<Vec<&str>>()
        .join("/");
    let list_content_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
        account, repository, subdir_path, branch
    );
    let _ = list_content(&list_content_url);
}

fn list_content(url: &str) -> Result<(), Error> {
    let client = Client::new();
    let mut response = client
        .get(url)
        .header("User-Agent", " ")
        .send()
        .expect("Failed to get response from the API");
    let mut body = String::new();
    response
        .read_to_string(&mut body)
        .expect("Failed to read response of the API");
    let item_list: Value = serde_json::from_str(&body)?;
    if let Some(array) = item_list.as_array() {
        for element in array {
            let file_type = element["type"].as_str().expect("No file type in the response items");
            if file_type == "dir" {
                let dir_url = element["url"].as_str().expect("Invalid sub-directory URL");
                let _ = list_content(dir_url);
            } else if file_type == "file" {
                let file_path = element["path"].as_str().expect("Invalid filepath in the sub-directory");
                let file_dir = Path::new(file_path)
                    .parent()
                    .expect("Invalid directory path of the file")
                    .to_str()
                    .expect("Unable to convert directory path to string");
                let file_url = element["download_url"].as_str().expect("Invalid download file URL");
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
    copy(
        &mut response.bytes().expect("Failed to get bytes").as_ref(),
        &mut writer,
    )
    .expect("Failed to copy data");
    writer.flush().expect("Failed to flush writer");
    println!(" {}", file_path);
}
