use anyhow::Result as AnyResult;
use clap::Arg;
use clap::ArgMatches;
use clap::{App, SubCommand};
use encoding_rs::Encoding;
use regex::Regex;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;
use std::str::FromStr;

/// Common options that apply for more than one command.
#[derive(Debug)]
pub struct GlobalConfig {
    pub path: PathBuf,
    pub no_confirm: bool,
}

#[derive(Debug)]
pub enum CommandConfig {
    Download(DownloadConfig),
    Rename(RenameConfig),
    Time(TimeConfig),
}

#[derive(Debug)]
pub struct DownloadConfig {
    pub url: String,
}

#[derive(Debug)]
pub struct RenameConfig {
    pub video_area: Option<Regex>,
    pub sub_area: Option<Regex>,
}

#[derive(Debug)]
pub struct TimeConfig {
    pub timing: i64,
    pub encoding: &'static Encoding,
    pub fps: f64,
}

impl GlobalConfig {
    pub fn parse() -> AnyResult<(GlobalConfig, CommandConfig)> {
        let matches = App::new("sub-batch")
            .version(env!("CARGO_PKG_VERSION"))
            .arg(
                Arg::with_name("path")
                    .long("path")
                    .short("p")
                    .takes_value(true)
                    .global(true)
                    .default_value(".")
                    .help("The path to download to and look for subs in."),
            )
            .arg(
                Arg::with_name("no_confirm")
                    .long("no-confirm")
                    .short("y")
                    .takes_value(false)
                    .help(
                        "If this flag is set sub-batch will not ask for any confirmation before \
                        applying operations.",
                    ),
            )
            .subcommand(
                SubCommand::with_name("download")
                    .about("Downloads all subs from a kitsunekko.net page")
                    .arg(
                        Arg::with_name("url")
                            .required(true)
                            .help("The kitsunekko.net url to download subs from.")
                    ),
            )
            .subcommand(
                SubCommand::with_name("rename")
                    .about("Renames subtitle files to match the corresponding video file")
                    .arg(
                        Arg::with_name("video_area")
                            .long("videoarea")
                            .short("va")
                            .takes_value(true)
                            .allow_hyphen_values(true)
                            .help(
                                "Specifies a regular expression that defines the part of the video \
                                filename where episode number should be extracted from.",
                            ),
                    )
                    .arg(
                        Arg::with_name("sub_area")
                            .long("subarea")
                            .short("sa")
                            .takes_value(true)
                            .allow_hyphen_values(true)
                            .help(
                                "Specifies a regular expression that defines the part of the \
                                subtitle filename where episode number should be extracted from.",
                            ),
                    )
            )
            .subcommand(
                SubCommand::with_name("time")
                    .about(
                        "Adjusts the timing of all subs. The value is specified in milliseconds, \
                         and can be negative",
                    )
                    .arg(
                        Arg::with_name("time")
                            .required(true)
                            .takes_value(true)
                            .allow_hyphen_values(true)
                    )
                    .arg(
                        Arg::with_name("encoding")
                            .long("encoding")
                            .short("e")
                            .takes_value(true)
                            .default_value("utf-8")
                            .help(
                                "Needed to parse text-based subtitle formats.",
                            ),
                    )
                    .arg(
                        Arg::with_name("fps")
                            .long("fps")
                            .takes_value(true)
                            .default_value("25")
                            .help(
                                "Needed for MicroDVD .sub files. Specifies the FPS that the video \
                                file is encoded in.",
                            ),
                    )
            )
            .get_matches();

        let (subcommand_name, subcommand_matches) = check_args(&matches);

        let command_config = match subcommand_name {
            "download" => CommandConfig::Download(DownloadConfig {
                url: subcommand_matches.value_of("url").unwrap().to_string(),
            }),
            "rename" => CommandConfig::Rename(RenameConfig {
                video_area: area(&subcommand_matches, "video_area")?,
                sub_area: area(&subcommand_matches, "sub_area")?,
            }),
            "time" => CommandConfig::Time(TimeConfig {
                timing: timing(&subcommand_matches)?,
                encoding: encoding(&subcommand_matches)?,
                fps: fps(&subcommand_matches)?,
            }),
            _ => unreachable!(),
        };

        Ok((
            GlobalConfig {
                path: matches.value_of("path").unwrap().into(),
                no_confirm: matches.is_present("no_confirm"),
            },
            command_config,
        ))
    }
}

fn check_args<'a>(matches: &'a ArgMatches) -> (&'a str, &'a ArgMatches<'a>) {
    // We require a subcommand so terminate if we don't get one (possible to do with clap?)
    if let (name, Some(sub)) = matches.subcommand() {
        (name, sub)
    } else {
        println!("{}", matches.usage().replace("[SUBCOMMAND]", "SUBCOMMAND"));
        std::process::exit(1);
    }
}

fn area(matches: &ArgMatches, key: &str) -> AnyResult<Option<Regex>> {
    Ok(if let Some(v) = matches.value_of(key) {
        Some(Regex::new(v)?)
    } else {
        None
    })
}

fn timing(matches: &ArgMatches) -> Result<i64, ParseIntError> {
    let v = matches.value_of("time").unwrap();
    i64::from_str(v)
}

fn encoding(matches: &ArgMatches) -> AnyResult<&'static Encoding> {
    let v = matches.value_of("encoding").unwrap();
    Encoding::for_label(v.as_bytes()).ok_or_else(|| anyhow!("invalid encoding"))
}

fn fps(matches: &ArgMatches) -> Result<f64, ParseFloatError> {
    let v = matches.value_of("fps").unwrap();
    f64::from_str(v)
}
