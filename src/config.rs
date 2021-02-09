use anyhow::Result as AnyResult;
use clap::ArgMatches;
use clap::{App, SubCommand};
use clap::{AppSettings, Arg};
use encoding_rs::Encoding;
use encoding_rs::UTF_8;
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
    Rename(RenameConfig),
    Time(TimeConfig),
    Alass(AlassConfig),
    Mpv,
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

#[derive(Debug)]
pub struct AlassConfig {
    pub flags: Vec<String>,
    pub video_area: Option<Regex>,
    pub sub_area: Option<Regex>,
    pub no_parallel: bool,
}

impl TimeConfig {
    pub fn timing(timing: i64) -> Self {
        Self {
            timing,
            encoding: UTF_8,
            fps: 25.0,
        }
    }
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
                    .help("The path to look for subs in."),
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
                SubCommand::with_name("rename")
                    .about("Renames subtitle files to match the corresponding video file.")
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
                    // allow_hyphen_values is broken with positional arguments so we have to
                    // set this for the entire subcommand. https://github.com/clap-rs/clap/issues/1437
                    .settings(&[AppSettings::AllowLeadingHyphen])
                    .about(
                        "Adjusts the timing of all subs. The value is specified in milliseconds, \
                         and can be negative.",
                    )
                    .arg(
                        Arg::with_name("time")
                            .required(true)
                            .takes_value(true)
                    )
                    .arg(
                        Arg::with_name("encoding")
                            .long("encoding")
                            .short("e")
                            .takes_value(true)
                            .help(
                                "Needed to parse text-based subtitle formats. Defaults to UTF-8.",
                            ),
                    )
                    .arg(
                        Arg::with_name("fps")
                            .long("fps")
                            .takes_value(true)
                            .help(
                                "Needed for MicroDVD .sub files. Specifies the FPS that the video \
                                file is encoded in. Defaults to 25.0",
                            ),
                    )
            )
            .subcommand(
                SubCommand::with_name("time-mpv")
                    .about("Adjusts the timing of all subs interactively using mpv. \
                            `mpv` must be installed on the system for this command to work.")
            )
            .subcommand(
                SubCommand::with_name("alass")
                    .settings(&[AppSettings::AllowLeadingHyphen])
                    .about(
                        "Adjusts the timing of all subs that are matched with a video file using \
                        `alass` (https://github.com/kaegi/alass). This can automatically fix \
                        wrong timings due to commercial breaks for example."
                    )
                    .arg(
                        Arg::with_name("flags")
                            .takes_value(true)
                            .help(
                                "A string of flags that is passed directly to alass for each \
                                subtitle/video adjustment. The arguments must be quoted so that \
                                they are interpreted as a single string, for example: \
                                \n\n  sub-batch alass \"--split-penalty 10\"",
                            ),
                    )
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
                    .arg(
                        Arg::with_name("no_parallel")
                            .long("nopar")
                            .takes_value(false)
                            .help(
                                "If this flag is set sub-batch will not execute alass in parallel."
                            ),
                    )
            )
            .get_matches();

        let (subcommand_name, subcommand_matches) = check_args(&matches);

        let command_config = match subcommand_name {
            "rename" => CommandConfig::Rename(RenameConfig {
                video_area: area(&subcommand_matches, "video_area")?,
                sub_area: area(&subcommand_matches, "sub_area")?,
            }),
            "time" => {
                let mut tc = TimeConfig::timing(timing(&subcommand_matches)?);
                if let Some(encoding) = encoding(&subcommand_matches) {
                    tc.encoding = encoding?;
                }
                if let Some(fps) = fps(&subcommand_matches) {
                    tc.fps = fps?;
                }
                CommandConfig::Time(tc)
            }
            "alass" => CommandConfig::Alass(AlassConfig {
                flags: alass_flags(&subcommand_matches),
                video_area: area(&subcommand_matches, "video_area")?,
                sub_area: area(&subcommand_matches, "sub_area")?,
                no_parallel: subcommand_matches.is_present("no_parallel"),
            }),
            "time-mpv" => CommandConfig::Mpv,
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

fn encoding(matches: &ArgMatches) -> Option<AnyResult<&'static Encoding>> {
    matches
        .value_of("encoding")
        .map(|v| Encoding::for_label(v.as_bytes()).ok_or_else(|| anyhow!("invalid encoding")))
}

fn fps(matches: &ArgMatches) -> Option<Result<f64, ParseFloatError>> {
    matches.value_of("fps").map(|v| f64::from_str(v))
}

fn alass_flags(matches: &ArgMatches) -> Vec<String> {
    match matches.value_of("flags") {
        None => Vec::new(),
        Some(flags) => flags.split_ascii_whitespace().map(str::to_string).collect(),
    }
}
