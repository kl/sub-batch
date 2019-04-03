mod actions;
mod config;
mod net;
mod sub_transformer;
mod util;

use crate::config::Config;
use crate::sub_transformer::Action;
use actions::*;
use sub_transformer::SubTransformerBuilder;
use util::AnyError;

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate lazy_static;

fn run() -> Result<(), AnyError> {
    let mut config = Config::parse()?;

    if let Some(ref url) = config.url {
        net::download_subs(url, &config.path)?;
    }

    let transformer = SubTransformerBuilder::default()
        .path(config.path.clone())
        .extensions(vec![".ssa", ".ass", ".sub", ".srt", ".idx"]) // same as subparse supports
        .video_area(config.video_area.take())
        .sub_area(config.sub_area.take())
        .actions(actions())
        .build()?;

    transformer.execute(&config)?;

    Ok(())
}

fn actions() -> Vec<Box<Action>> {
    // Order is important here. If Renamer were to run first it would break TimingAdjuster
    // because they both operate on the same file paths.
    vec![Box::new(TimingAdjuster::new()), Box::new(Renamer::new())]
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
    println!("Ok");
}
