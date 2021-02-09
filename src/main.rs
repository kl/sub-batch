mod config;
mod scanner;
mod commands {
    pub mod alass;
    pub mod mpv;
    pub mod rename;
    pub mod time;
    mod util;
}
use crate::commands::mpv::MpvCommand;
use crate::config::{CommandConfig, GlobalConfig};
use alass::AlassCommand;
use anyhow::Result as AnyResult;
use commands::*;
use crossterm::terminal;
use std::process;
use CommandConfig::*;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

fn main() {
    setup_signal_handler();

    let catch = std::panic::catch_unwind(|| {
        if let Err(e) = run() {
            let _ = terminal::disable_raw_mode();
            eprintln!("\nerror: {}", e);
            process::exit(1);
        }
    });

    let _ = terminal::disable_raw_mode();
    if catch.is_err() {
        process::exit(1);
    }
}

fn setup_signal_handler() {
    let handler = ctrlc::set_handler(|| {
        let _ = terminal::disable_raw_mode();
        process::exit(1);
    });

    if handler.is_err() {
        eprintln!("warning: failed to set Ctrl-C handler");
    }
}

fn run() -> AnyResult<()> {
    let (global_config, cmd_config) = GlobalConfig::parse()?;

    // Delegate to the right command.
    match cmd_config {
        Rename(c) => rename::run(&global_config, c),
        Time(c) => time::run(&global_config, c),
        Alass(c) => AlassCommand::new(global_config, c).run(),
        Mpv => MpvCommand::new(global_config).run(),
    }?;

    Ok(())
}
