use crate::commands::util;
use crate::config::{GlobalConfig, RenameConfig};
use crate::scanner;
use crate::scanner::{MatchInfo, ScanOptions};
use anyhow::Result as AnyResult;
use std::fs;

pub fn run(global_conf: &GlobalConfig, conf: RenameConfig) -> AnyResult<()> {
    let matches = scanner::scan(ScanOptions {
        path: &global_conf.path,
        sub_area: conf.sub_area.as_ref(),
        video_area: conf.video_area.as_ref(),
        sub_filter: global_conf.sub_filter.as_ref(),
        video_filter: global_conf.video_filter.as_ref(),
    })?;

    util::validate_sub_and_file_matches_ignore_extensions(global_conf, &matches)?;

    // Filter out subs that already have the same name as their video file
    let renames: Vec<MatchInfo> = matches
        .into_iter()
        .filter(|re| re.matched.sub_file_part != re.matched.vid_file_part)
        .collect();

    if renames.is_empty() {
        println!("all subtitles are already renamed");
        return Ok(());
    }

    if global_conf.no_confirm || util::ask_user_ok(&renames, true)? {
        for rename in renames.iter() {
            let new_name = rename
                .matched
                .vid_path
                .with_extension(&rename.matched.sub_ext_part);
            fs::rename(&rename.matched.sub_path, new_name)?;
        }
    }

    Ok(())
}
