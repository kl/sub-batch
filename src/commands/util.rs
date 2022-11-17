use crate::config::GlobalConfig;
use crate::scanner::SubAndFile;
use anyhow::Result as AnyResult;
use core::result::Result::Ok;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

// these are the formats that subparse currently supports
pub static SUBPARSE_SUPPORTED_SUBTITLE_FORMATS: &[&str] = &["ssa", "ass", "sub", "srt", "idx"];

pub fn ask_user_ok(renames: &[SubAndFile]) -> AnyResult<bool> {
    for rename in renames.iter() {
        println!(
            "{} -> {}",
            rename.sub_path.to_string_lossy(),
            rename.file_path.to_string_lossy()
        );
    }
    println!("Ok? (y/n)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.to_lowercase().starts_with('y'))
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
    matches: &[SubAndFile],
) -> AnyResult<()> {
    validate_sub_and_file_matches_ignore_extensions(global_conf, matches)?;
    let sub_files: Vec<&PathBuf> = matches.iter().map(|m| &m.sub_path).collect();
    validate_sub_extensions(&sub_files)?;
    Ok(())
}

pub fn validate_sub_and_file_matches_ignore_extensions(
    global_conf: &GlobalConfig,
    matches: &[SubAndFile],
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
            .and_then(&OsStr::to_str)
            .map(|ext| SUBPARSE_SUPPORTED_SUBTITLE_FORMATS.contains(&ext))
            == Some(true)
    })
}
