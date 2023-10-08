use crate::config::GlobalConfig;
use crate::scanner::MatchInfo;
use anyhow::Result as AnyResult;
use core::result::Result::Ok;
use crossterm::style::Stylize;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

// these are the formats that subparse currently supports
pub static SUBPARSE_SUPPORTED_SUBTITLE_FORMATS: &[&str] = &["ssa", "ass", "sub", "srt", "idx"];

pub fn ask_user_ok(renames: &[MatchInfo], highlight: bool) -> AnyResult<bool> {
    for rename in renames.iter() {
        let (sub_before, sub_match, sub_after) = rename.sub_match_parts();
        let sub_ext = &rename.matched.sub_ext_part.to_string_lossy();
        let (vid_before, vid_match, vid_after) = rename.vid_match_parts();
        let vid_ext = rename.matched.vid_ext_part.as_ref().map(|e| e.to_string_lossy());

        print!("{sub_before}");
        if highlight {
            print!("{}", sub_match.black().bold().on_yellow());
        } else {
            print!("{}", sub_match);
        }
        print!("{sub_after}.{sub_ext} -> ");

        print!("{vid_before}");
        if highlight {
            print!("{}", vid_match.black().bold().on_yellow());
        } else {
            print!("{}", vid_match);
        }
        print!("{vid_after}");
        if let Some(vid_ext) = vid_ext {
            print!(".{vid_ext}");
        }

        println!();
    }
    println!("Ok? (Y/n)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.split_whitespace().next().is_none() || input.to_lowercase().starts_with('y'))
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
