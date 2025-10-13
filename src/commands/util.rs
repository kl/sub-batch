use crate::config::GlobalConfig;
use crate::scanner::{MatchInfo, MatchInfoType};
use anyhow::Result as AnyResult;
use core::result::Result::Ok;
use crossterm::style::Stylize;
use regex::Regex;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

// these are the formats that subparse currently supports
pub static SUBPARSE_SUPPORTED_SUBTITLE_FORMATS: &[&str] = &["ssa", "ass", "sub", "srt", "idx"];

#[derive(Debug, PartialEq)]
pub enum AskMatchAnswer {
    Yes,
    No,
    EditSubtitleRegex,
    EditVideoRegex,
}

pub fn ask_match_is_ok(
    renames: &[MatchInfo],
    sub_area_regex: Option<&Regex>,
    video_area_regex: Option<&Regex>,
    color: bool,
    print_identical: bool,
    mut line_editor: Option<&mut DefaultEditor>,
) -> AnyResult<AskMatchAnswer> {
    fn print_file_parts(
        file_name: &str,
        area_range: &Option<Range<usize>>,
        number_range: &Range<usize>,
        color: bool,
    ) {
        let (before, area_left, num, area_right, after) = if let Some(area) = area_range {
            let before = &file_name[0..area.start];
            let area_left = &file_name[area.start..number_range.start];
            let num = &file_name[number_range.start..number_range.end];
            let area_right = &file_name[number_range.end..area.end];
            let after = &file_name[area.end..];
            (before, area_left, num, area_right, after)
        } else {
            let before = &file_name[0..number_range.start];
            let num = &file_name[number_range.start..number_range.end];
            let after = &file_name[number_range.end..];
            (before, "", num, "", after)
        };

        print!("{before}");
        if color {
            print!("{}", area_left.black().bold().on_magenta());
            print!("{}", num.black().bold().on_yellow());
            print!("{}", area_right.black().bold().on_magenta());
        } else {
            print!("{}", area_left);
            print!("{}", num);
            print!("{}", area_right);
        }
        print!("{after}");
    }

    if renames.is_empty() {
        return Ok(AskMatchAnswer::No);
    }

    let longest_sub_length = renames
        .iter()
        .map(|sub| sub.sub_file_name.len())
        .max_by(|a, b| a.cmp(b))
        .unwrap();

    for rename in renames.iter() {
        let padding = str::repeat(" ", longest_sub_length - rename.sub_file_name.len());

        let MatchInfoType::NumberMatch {
            sub_number_range,
            video_number_range,
            sub_match_area,
            video_match_area,
        } = &rename.match_type
        else {
            if print_identical {
                println!(
                    "{}{} -> {}",
                    rename.sub_file_name, padding, rename.video_file_name
                );
            }
            continue;
        };

        print_file_parts(
            &rename.sub_file_name,
            sub_match_area,
            sub_number_range,
            color,
        );

        print!("{}", padding);
        print!(" -> ");

        print_file_parts(
            &rename.video_file_name,
            video_match_area,
            video_number_range,
            color,
        );
        println!();
    }

    println!(
        "\n[s = edit subtitle regex (current: {}), v = edit video regex (current: {})]",
        sub_area_regex
            .map(|r| r.to_string())
            .unwrap_or("none".to_string()),
        video_area_regex
            .map(|r| r.to_string())
            .unwrap_or("none".to_string()),
    );

    let prompt = "Ok? (Y/n): ";
    let input = if let Some(ref mut editor) = line_editor {
        let readline = editor.readline(prompt);
        match readline {
            Ok(line) => line.to_lowercase(),
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => return Ok(AskMatchAnswer::No),
            Err(err) => bail!(err),
        }
    } else {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.to_lowercase()
    };

    Ok(
        if input.split_whitespace().next().is_none() || input.starts_with('Y') {
            AskMatchAnswer::Yes
        } else if input.starts_with('s') {
            AskMatchAnswer::EditSubtitleRegex
        } else if input.starts_with('v') {
            AskMatchAnswer::EditVideoRegex
        } else {
            AskMatchAnswer::No
        },
    )
}

pub fn validate_sub_matches(global_conf: &GlobalConfig, matches: &[PathBuf]) -> AnyResult<()> {
    if matches.is_empty() {
        bail!("found no subtitle files in {:?}", &global_conf.path);
    }
    validate_sub_extensions(matches)?;
    Ok(())
}

pub fn validate_sub_and_file_matches(
    global_conf: &GlobalConfig,
    matches: &[MatchInfo],
) -> AnyResult<()> {
    validate_sub_and_file_matches_ignore_extensions(global_conf, matches)?;
    let sub_files: Vec<&PathBuf> = matches.iter().map(|m| &m.sub_path).collect();
    validate_sub_extensions(&sub_files)?;
    Ok(())
}

pub fn validate_sub_and_file_matches_ignore_extensions(
    global_conf: &GlobalConfig,
    matches: &[MatchInfo],
) -> AnyResult<()> {
    if matches.is_empty() {
        bail!(
            "found no video/subtitle file pairs in {:?}",
            &global_conf.path
        );
    }
    Ok(())
}

fn validate_sub_extensions(sub_files: &[impl AsRef<Path>]) -> AnyResult<()> {
    if !has_subparse_supported_subtitle_formats(sub_files) {
        bail!(
            "command supports only the following subtitle formats: {:?}",
            SUBPARSE_SUPPORTED_SUBTITLE_FORMATS
        );
    }
    Ok(())
}

fn has_subparse_supported_subtitle_formats(matches: &[impl AsRef<Path>]) -> bool {
    matches.iter().all(|m| {
        m.as_ref()
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| SUBPARSE_SUPPORTED_SUBTITLE_FORMATS.contains(&ext))
            == Some(true)
    })
}

pub fn get_user_regex(
    prompt: &str,
    mut line_editor: Option<&mut DefaultEditor>,
) -> AnyResult<Option<Regex>> {
    if let Some(ref mut editor) = line_editor {
        let readline = editor.readline(prompt);
        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_str())?;
                if let Ok(regex) = Regex::new(line.as_str()) {
                    Ok(Some(regex))
                } else {
                    get_user_regex("invalid regex, try again: ", line_editor)
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => Ok(None),
            Err(err) => bail!(err),
        }
    } else {
        print!("{prompt}");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.pop();
        if let Ok(regex) = Regex::new(&input) {
            Ok(Some(regex))
        } else {
            get_user_regex("invalid regex, try again: ", line_editor)
        }
    }
}
