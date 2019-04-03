use crate::util::AnyError;
use clap::App;
use clap::Arg;
use clap::ArgMatches;
use encoding_rs::Encoding;
use regex::Regex;
use std::env;
use std::num::{ParseFloatError, ParseIntError};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct Config {
    pub url: Option<String>,
    pub rename: bool,
    pub path: PathBuf,
    pub video_area: Option<Regex>,
    pub sub_area: Option<Regex>,
    pub timing: Option<i64>,
    pub encoding: &'static Encoding,
    pub fps: f64,
}

impl Config {
    pub fn parse() -> Result<Self, AnyError> {
        let matches = App::new("subs")
            .version(env!("CARGO_PKG_VERSION"))
            .arg(
                Arg::with_name("url")
                    .takes_value(true)
                    .help("The kitsunekko.net url to download subs from. May be omitted."),
            )
            .arg(
                Arg::with_name("rename")
                    .long("rename")
                    .short("r")
                    .takes_value(false)
                    .help("If subs should be renamed to match the corresponding video file."),
            )
            .arg(
                Arg::with_name("path")
                    .long("path")
                    .short("p")
                    .takes_value(true)
                    .default_value(".")
                    .help("The path to download to and look for subs in."),
            )
            .arg(
                Arg::with_name("videoarea")
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
                Arg::with_name("subarea")
                    .long("subarea")
                    .short("sa")
                    .takes_value(true)
                    .allow_hyphen_values(true)
                    .help(
                        "Specifies a regular expression that defines the part of the subtitle \
                         filename where episode number should be extracted from.",
                    ),
            )
            .arg(
                Arg::with_name("timing")
                    .long("timing")
                    .short("t")
                    .takes_value(true)
                    .allow_hyphen_values(true)
                    .help(
                        "Adjusts the timing of all subs. The value is specified in milliseconds, \
                         and can be negative.",
                    ),
            )
            .arg(
                Arg::with_name("encoding")
                    .long("encoding")
                    .short("e")
                    .takes_value(true)
                    .default_value("utf-8")
                    .help(
                        "Needed to parse text-based subtitle formats (only needed when adjusting \
                         timing).",
                    ),
            )
            .arg(
                Arg::with_name("fps")
                    .long("fps")
                    .takes_value(true)
                    .default_value("25")
                    .help(
                        "Needed for MicroDVD .sub files. Specifies the FPS that the video file \
                         is encoded in (only needed when adjusting timing).",
                    ),
            )
            .get_matches();

        check_args(&matches);

        Ok(Config {
            url: matches.value_of("url").map(str::to_string),
            rename: matches.is_present("rename"),
            path: matches.value_of("path").unwrap().into(),
            video_area: area(&matches, "videoarea")?,
            sub_area: area(&matches, "subarea")?,
            timing: timing(&matches)?,
            encoding: encoding(&matches)?,
            fps: fps(&matches)?,
        })
    }
}

fn check_args(matches: &ArgMatches) {
    // We require at least one argument so terminate if we don't get one (possible to do with clap?)
    if env::args().len() < 2 {
        println!("{}", matches.usage());
        std::process::exit(1);
    }
}

fn area(matches: &ArgMatches, key: &str) -> Result<Option<Regex>, AnyError> {
    Ok(if let Some(v) = matches.value_of(key) {
        Some(Regex::new(v)?)
    } else {
        None
    })
}

fn timing(matches: &ArgMatches) -> Result<Option<i64>, ParseIntError> {
    Ok(if let Some(v) = matches.value_of("timing") {
        let ms = i64::from_str(v)?;
        Some(ms)
    } else {
        None
    })
}

fn encoding(matches: &ArgMatches) -> Result<&'static Encoding, String> {
    let v = matches.value_of("encoding").unwrap();
    Encoding::for_label(v.as_bytes()).ok_or_else(|| "invalid encoding".to_string())
}

fn fps(matches: &ArgMatches) -> Result<f64, ParseFloatError> {
    let v = matches.value_of("fps").unwrap();
    f64::from_str(v)
}
