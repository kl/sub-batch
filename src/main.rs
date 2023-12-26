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
use crate::commands::rename::RenameCommand;
use crate::commands::time::TimeCommand;
use crate::config::{CommandConfig, GlobalConfig};
use alass::AlassCommand;
use anyhow::Result as AnyResult;
use commands::*;
use crossterm::{cursor, terminal, ExecutableCommand};
use std::process;
use CommandConfig::*;

#[macro_use]
extern crate anyhow;

fn main() {
    setup_signal_handler();

    let catch = std::panic::catch_unwind(|| {
        if let Err(e) = run() {
            restore_terminal();
            eprintln!("\nerror: {}", e);
            process::exit(1);
        }
    });

    restore_terminal();
    if catch.is_err() {
        process::exit(1);
    }
}

fn setup_signal_handler() {
    let handler = ctrlc::set_handler(|| {
        restore_terminal();
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
        Rename(c) => RenameCommand::new(&global_config, c).run(),
        Time(c) => TimeCommand::new(&global_config, c).run(),
        Alass(c) => AlassCommand::new(&global_config, c).run(),
        Mpv => MpvCommand::new(&global_config).run(),
    }?;

    Ok(())
}

fn restore_terminal() {
    let _ = std::io::stdout().execute(cursor::Show);
    let _ = terminal::disable_raw_mode();
}
