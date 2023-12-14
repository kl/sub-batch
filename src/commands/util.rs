use crate::config::GlobalConfig;
use crate::scanner::MatchInfo;
use anyhow::Result as AnyResult;
use core::result::Result::Ok;
use crossterm::style::Stylize;
use regex::{Match, Regex};
use std::ffi::OsStr;
use std::io::{self, Write};
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
    sub_area: Option<&Regex>,
    vid_area: Option<&Regex>,
    highlight: bool,
) -> AnyResult<AskMatchAnswer> {
    fn print_highlights(
        matcher: Option<Match>,
        before: &str,
        target: &str,
        after: &str,
        highlight: bool,
    ) {
        let (before_no_hl, before_hl, after_hl, after_no_hl) = if let Some(matcher) = matcher {
            let start = matcher.range().start;
            let end = matcher.range().end - (before.len() + target.len());
            (
                &before[0..start],
                &before[start..],
                &after[..end],
                &after[end..],
            )
        } else {
            // No area regex match means no extra highlights are needed
            (before, "", "", after)
        };

        print!("{before_no_hl}");
        if highlight {
            print!("{}", before_hl.black().bold().on_grey());
            print!("{}", target.black().bold().on_yellow());
            print!("{}", after_hl.black().bold().on_grey());
        } else {
            print!("{}", before_hl);
            print!("{}", target);
            print!("{}", after_hl);
        }
        print!("{after_no_hl}");
    }

    for rename in renames.iter() {
        let (sub_before, sub_match, sub_after) = rename.sub_match_parts();
        let sub_ext = &rename.matched.sub_ext_part.to_string_lossy();
        let (vid_before, vid_match, vid_after) = rename.vid_match_parts();
        let vid_ext = rename
            .matched
            .vid_ext_part
            .as_ref()
            .map(|e| e.to_string_lossy());

        let sub_area_match = sub_area.and_then(|r| r.find(&rename.matched.sub_file_part));
        let vid_area_match = vid_area.and_then(|r| r.find(&rename.matched.vid_file_part));

        print_highlights(sub_area_match, sub_before, sub_match, sub_after, highlight);
        print!(".{sub_ext} -> ");
        print_highlights(vid_area_match, vid_before, vid_match, vid_after, highlight);
        if let Some(vid_ext) = vid_ext {
            print!(".{vid_ext}");
        }
        println!();
    }

    println!(
        "\n[s = edit subtitle regex (current: {}), v = edit video regex (current: {})]",
        sub_area
            .map(|r| r.to_string())
            .unwrap_or("None".to_string()),
        vid_area
            .map(|r| r.to_string())
            .unwrap_or("None".to_string()),
    );
    print!("Ok? (Y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    input = input.to_lowercase();

    Ok(
        if input.split_whitespace().next().is_none() || input.starts_with('y') {
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
    let sub_files: Vec<&PathBuf> = matches.iter().map(|m| &m.matched.sub_path).collect();
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

pub fn get_user_regex(prompt: &str) -> AnyResult<Regex> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    input.pop();
    if let Ok(regex) = Regex::new(&input) {
        Ok(regex)
    } else {
        get_user_regex("invalid regex, try again: ")
    }
}
