use crate::scanner::{AreaScan, SecondaryExtensionPolicy};
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
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    pub path: PathBuf,
    pub confirm: bool,
    pub color: bool,
    pub sub_filter: Option<Regex>,
    pub video_filter: Option<Regex>,
}

#[derive(Debug)]
pub enum CommandConfig {
    Rename(MatchFilesConfig),
    Time(TimeConfig),
    Alass(AlassConfig),
    Mpv,
}

#[derive(Debug, Clone)]
pub struct MatchFilesConfig {
    pub sub_area: Option<Regex>,
    pub sub_area_scan: AreaScan,
    pub video_area: Option<Regex>,
    pub video_area_scan: AreaScan,
    pub secondary_ext_policy: SecondaryExtensionPolicy,
}

#[derive(Debug)]
pub struct TimeConfig {
    pub timing: i64,
    pub encoding: &'static Encoding,
    pub fps: f64,
}

#[derive(Debug, Clone)]
pub struct AlassConfig {
    pub flags: Vec<String>,
    pub no_parallel: bool,
    pub match_config: MatchFilesConfig,
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
                    .help("The path to look for subs/videos in."),
            )
            .arg(
                Arg::with_name("no_confirm")
                    .long("no-confirm")
                    .short("y")
                    .takes_value(false)
                    .help(
                        "If set no confirmation prompt is shown before applying file operations.",
                    ),
            )
            .arg(
                Arg::with_name("no_color")
                    .long("no-color")
                    .takes_value(false)
                    .help("If set text colors are not used for highlights."),
            )
            .arg(
                Arg::with_name("filter_sub")
                    .long("filter-sub")
                    .short("s")
                    .takes_value(true)
                    .help(
                        "Only subtiles with file names that match this regex will be targeted by \
                        any of the SUBCOMMANDS.",
                    ),
            )
            .arg(
                Arg::with_name("filter_video")
                    .long("filter-video")
                    .short("v")
                    .takes_value(true)
                    .help(
                        "Only videos with file names that match this regex will be targeted by \
                        any of the SUBCOMMANDS.",
                    ),
            )
            .subcommand(
                SubCommand::with_name("rename")
                    .about("Renames subtitle files to match the corresponding video file.")
                    .common_match_args(),
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
                    .arg(Arg::with_name("time").required(true).takes_value(true))
                    .arg(
                        Arg::with_name("encoding")
                            .long("encoding")
                            .short("e")
                            .takes_value(true)
                            .help(
                                "Needed to parse text-based subtitle formats. Defaults to UTF-8.",
                            ),
                    )
                    .arg(Arg::with_name("fps").long("fps").takes_value(true).help(
                        "Needed for MicroDVD .sub files. Specifies the FPS that the video \
                                file is encoded in. Defaults to 25.0",
                    )),
            )
            .subcommand(SubCommand::with_name("time-mpv").about(
                "Adjusts the timing of all subs interactively using mpv. `mpv` must be installed.",
            ))
            .subcommand(
                SubCommand::with_name("alass")
                    .settings(&[AppSettings::AllowLeadingHyphen])
                    .about(
                        "Adjusts the timing of all subs that are matched with a video file using \
                        `alass` (https://github.com/kaegi/alass). This can automatically fix \
                        wrong timings.",
                    )
                    .common_match_args()
                    .arg(Arg::with_name("flags").takes_value(true).help(
                        "A string of flags that is passed directly to alass for each \
                                subtitle/video adjustment. The arguments must be quoted so that \
                                they are interpreted as a single string, for example: \
                                \n\n  sub-batch alass \"--split-penalty 10\"",
                    ))
                    .arg(
                        Arg::with_name("no_parallel")
                            .long("nopar")
                            .takes_value(false)
                            .help(
                                "If this flag is set sub-batch will not execute alass in parallel.",
                            ),
                    ),
            )
            .get_matches();

        let (subcommand_name, subcommand_matches) = check_args(&matches);

        let sub_area_scan = if subcommand_matches.is_present("reverse")
            || subcommand_matches.is_present("reverse_sub")
        {
            AreaScan::Reverse
        } else {
            AreaScan::Normal
        };

        let video_area_scan = if subcommand_matches.is_present("reverse")
            || subcommand_matches.is_present("reverse_video")
        {
            AreaScan::Reverse
        } else {
            AreaScan::Normal
        };

        let command_config = match subcommand_name {
            "rename" => CommandConfig::Rename(MatchFilesConfig {
                sub_area: regex_arg(subcommand_matches, "sub_area")?,
                sub_area_scan,
                video_area: regex_arg(subcommand_matches, "video_area")?,
                video_area_scan,
                secondary_ext_policy: secondary_ext_policy(subcommand_matches),
            }),
            "time" => {
                let mut tc = TimeConfig::timing(timing(subcommand_matches)?);
                if let Some(encoding) = encoding(subcommand_matches) {
                    tc.encoding = encoding?;
                }
                if let Some(fps) = fps(subcommand_matches) {
                    tc.fps = fps?;
                }
                CommandConfig::Time(tc)
            }
            "alass" => CommandConfig::Alass(AlassConfig {
                flags: alass_flags(subcommand_matches),
                no_parallel: subcommand_matches.is_present("no_parallel"),
                match_config: MatchFilesConfig {
                    sub_area: regex_arg(subcommand_matches, "sub_area")?,
                    sub_area_scan,
                    video_area: regex_arg(subcommand_matches, "video_area")?,
                    video_area_scan,
                    secondary_ext_policy: secondary_ext_policy(subcommand_matches),
                },
            }),
            "time-mpv" => CommandConfig::Mpv,
            _ => unreachable!(),
        };

        Ok((
            GlobalConfig {
                path: matches.value_of("path").unwrap().into(),
                confirm: !matches.is_present("no_confirm"),
                color: !matches.is_present("no_color"),
                sub_filter: regex_arg(&matches, "filter_sub")?,
                video_filter: regex_arg(&matches, "filter_video")?,
            },
            command_config,
        ))
    }
}

fn secondary_ext_policy(matches: &ArgMatches) -> SecondaryExtensionPolicy {
    if matches.is_present("secondary_ext_always") {
        SecondaryExtensionPolicy::Always
    } else if matches.is_present("secondary_ext_never") {
        SecondaryExtensionPolicy::Never
    } else {
        SecondaryExtensionPolicy::Maybe
    }
}

trait CommonMatchArgs {
    fn common_match_args(self) -> Self;
}

impl<'a, 'b> CommonMatchArgs for App<'a, 'b> {
    fn common_match_args(self) -> Self {
        self.arg(
            Arg::with_name("video_area")
                .long("videoarea")
                .short("v")
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
                .short("s")
                .takes_value(true)
                .allow_hyphen_values(true)
                .help(
                    "Specifies a regular expression that defines the part of the \
                    subtitle filename where episode number should be extracted from.",
                ),
        )
        .arg(
            Arg::with_name("reverse")
                .long("rev")
                .takes_value(false)
                .help("Looks for the sub and video numbers starting from the end of the area."),
        )
        .arg(
            Arg::with_name("reverse_sub")
                .long("rs")
                .takes_value(false)
                .help("Looks for the sub numbers starting from the end of the area."),
        )
        .arg(
            Arg::with_name("reverse_video")
                .long("rv")
                .takes_value(false)
                .help("Looks for the video numbers starting from the end of the area."),
        )
        .arg(
            Arg::with_name("secondary_ext_always")
                .long("sec-always")
                .takes_value(false)
                .help(
                    "By default, secondary extensions (i.e. \".jp\" in the file \"sub.jp.srt\" \
                    are only treated as part of the extension and not part of the file name if \
                    the following two conditions are true: \
                    1. The secondary extension is no longer than 3 characters long.
                    2. The secondary extension does not contain any numbers.
                    The length restriction exists because it matches the default behavior of 
                    mpv, and the number restriction because if the secondary extension contains 
                    a number, that number may be the number that is used to match the video file.
                    If this flag is set these two restrictions no longer apply and ANY 
                    secondary extension is treated as part of the file extension.",
                ),
        )
        .arg(
            Arg::with_name("secondary_ext_never")
                .long("sec-never")
                .takes_value(false)
                .help(
                    "By default, secondary extensions (i.e. \".jp\" in the file \"sub.jp.srt\" \
                    are only treated as part of the extension and not part of the file name if \
                    the following two conditions are true: \
                    1. The secondary extension is no longer than 3 characters long.
                    2. The secondary extension does not contain any numbers.
                    The length restriction exists because it matches the default behavior of 
                    mpv, and the number restriction because if the secondary extension contains 
                    a number, that number may be the number that is used to match the video file.
                    If this flag is set secondary extensions are ignored and always treated as 
                    part of the file stem.",
                ),
        )
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

fn regex_arg(matches: &ArgMatches, key: &str) -> AnyResult<Option<Regex>> {
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
    matches.value_of("fps").map(f64::from_str)
}

fn alass_flags(matches: &ArgMatches) -> Vec<String> {
    match matches.value_of("flags") {
        None => Vec::new(),
        Some(flags) => flags.split_ascii_whitespace().map(str::to_string).collect(),
    }
}
