mod config;
mod scanner;
mod commands {
    pub mod download;
    pub mod rename;
    pub mod time;
}
use crate::config::{CommandConfig, GlobalConfig};
use anyhow::Result as AnyResult;
use commands::*;
use CommandConfig::*;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> AnyResult<()> {
    let (global_config, cmd_config) = GlobalConfig::parse()?;

    // Delegate to the right command.
    match cmd_config {
        Download(c) => download::run(global_config, c),
        Rename(c) => rename::run(global_config, c),
        Time(c) => time::run(global_config, c),
    }?;

    Ok(())
}
