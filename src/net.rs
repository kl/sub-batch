use crate::util::AnyError;
use rayon::prelude::*;
use regex::Regex;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;

pub fn download_subs(url: &str, path: impl AsRef<Path>) -> Result<(), AnyError> {
    let path = path.as_ref();
    let text = download_string(url)?;

    let links_start = text
        .split("flisttable")
        .last()
        .ok_or("failed to find 'flisttable' in html")?;
    let links: &str = links_start.split("</table>").next().unwrap();

    let re = Regex::new("href=\"(.+?)\"").unwrap();

    let subs: Vec<_> = re
        .captures_iter(links)
        .map(|m| m.get(1).unwrap().as_str())
        .collect();

    let _: Vec<_> = subs
        .par_iter()
        .map(|sub| {
            let url = format!("https://kitsunekko.net/{}", sub);
            let text = download_bytes(&url).unwrap();
            let filename = sub.split('/').last().unwrap();
            let file_path = path.join(filename);
            println!("{:?}", file_path);
            std::fs::write(file_path, text).unwrap();
        })
        .collect();

    Ok(())
}

fn download_string(url: &str) -> Result<String, AnyError> {
    println!("downloading url: {}", url);
    let mut resp = reqwest::get(url)?;

    if !resp.status().is_success() {
        return Err(Box::new(IoError::new(
            ErrorKind::Other,
            format!("expected 200 OK, got: {}", resp.status()),
        )));
    }

    Ok(resp.text()?)
}

fn download_bytes(url: &str) -> Result<Vec<u8>, AnyError> {
    println!("downloading url: {}", url);
    let mut resp = reqwest::get(url)?;

    if !resp.status().is_success() {
        return Err(Box::new(IoError::new(
            ErrorKind::Other,
            format!("expected 200 OK, got: {}", resp.status()),
        )));
    }

    let mut r = Vec::new();
    resp.read_to_end(&mut r)?;
    Ok(r)
}
