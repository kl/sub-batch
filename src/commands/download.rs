use crate::config::{DownloadConfig, GlobalConfig};
use anyhow::Result as AnyResult;
use bytes::Bytes;
use rayon::prelude::*;
use regex::Regex;

pub fn run(global_conf: GlobalConfig, conf: DownloadConfig) -> AnyResult<()> {
    let path = &global_conf.path;
    let text = download_string(&conf.url)?;

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
            let bytes = download_bytes(&url).unwrap();
            let filename = sub.split('/').last().unwrap();
            let file_path = path.join(filename);
            println!("{:?}", file_path);
            std::fs::write(file_path, bytes).unwrap();
        })
        .collect();

    Ok(())
}

fn download_string(url: &str) -> AnyResult<String> {
    println!("downloading url: {}", url);
    let resp = reqwest::blocking::get(url)?;

    if !resp.status().is_success() {
        bail!("expected 200 OK, got: {}", resp.status());
    }

    Ok(resp.text()?)
}

fn download_bytes(url: &str) -> AnyResult<Bytes> {
    println!("downloading url: {}", url);
    let resp = reqwest::blocking::get(url)?;

    if !resp.status().is_success() {
        bail!("expected 200 OK, got: {}", resp.status());
    }

    Ok(resp.bytes()?)
}
