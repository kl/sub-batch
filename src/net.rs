use anyhow::Result as AnyResult;
use rayon::prelude::*;
use regex::Regex;
use std::io::Read;
use std::path::Path;

pub fn download_subs(url: &str, path: impl AsRef<Path>) -> AnyResult<()> {
    let path = path.as_ref();
    let text = download_string(url)?;

    let links_start = text
        .split("flisttable")
        .last()
        .ok_or_else(|| anyhow!("failed to find 'flisttable' in html"))?;

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

fn download_string(url: &str) -> AnyResult<String> {
    println!("downloading url: {}", url);
    let mut resp = reqwest::get(url)?;

    if !resp.status().is_success() {
        bail!("expected 200 OK, got: {}", resp.status());
    }

    Ok(resp.text()?)
}

fn download_bytes(url: &str) -> AnyResult<Vec<u8>> {
    println!("downloading url: {}", url);
    let mut resp = reqwest::get(url)?;

    if !resp.status().is_success() {
        bail!("expected 200 OK, got: {}", resp.status());
    }

    let mut r = Vec::new();
    resp.read_to_end(&mut r)?;
    Ok(r)
}
