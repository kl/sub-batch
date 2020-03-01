mod actions;
mod config;
mod net;
mod sub_transformer;

use crate::config::Config;
use crate::sub_transformer::Action;
use actions::*;
use anyhow::Result as AnyResult;
use sub_transformer::SubTransformerBuilder;

#[macro_use]
extern crate derive_builder;

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
        .build()
        .map_err(|e| anyhow!("failed to create SubTransformer: {:?}", e))?;

    transformer.execute(&config)?;

    Ok(())
}

fn actions() -> Vec<Box<dyn Action>> {
    // Order is important here. If Renamer were to run first it would break TimingAdjuster
    // because they both operate on the same file paths.
    vec![Box::new(TimingAdjuster::new()), Box::new(Renamer::new())]
}
