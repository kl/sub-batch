use crate::commands::util;
use crate::commands::util::AskMatchAnswer;
use crate::config::{GlobalConfig, RenameConfig};
use crate::scanner;
use crate::scanner::{MatchInfo, ScanOptions};
use anyhow::Result as AnyResult;
use std::fs;

pub fn run(global_conf: &GlobalConfig, mut conf: RenameConfig) -> AnyResult<()> {
    let matches = scanner::scan(ScanOptions::from_global_conf(
        global_conf,
        conf.sub_area.as_ref(),
        conf.sub_area_scan,
        conf.video_area.as_ref(),
        conf.video_area_scan,
    ))?;

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

    if global_conf.no_confirm {
        rename_subtitles(&renames)?;
        return Ok(());
    }

    match util::ask_match_is_ok(
        &renames,
        conf.sub_area.as_ref(),
        conf.video_area.as_ref(),
        true,
    )? {
        AskMatchAnswer::Yes => rename_subtitles(&renames)?,
        AskMatchAnswer::EditSubtitleRegex => loop {
            conf.sub_area = Some(util::get_user_regex("input new subtitle area regex: ")?);
            if let Err(e) = run(global_conf, conf.clone()) {
                println!("error: {}", e);
            } else {
                break;
            }
        },
        AskMatchAnswer::EditVideoRegex => loop {
            conf.video_area = Some(util::get_user_regex("input new video area regex: ")?);
            if let Err(e) = run(global_conf, conf.clone()) {
                println!("error: {}", e);
            } else {
                break;
            }
        },
        AskMatchAnswer::No => {}
    }
    Ok(())
}

fn rename_subtitles(renames: &[MatchInfo]) -> AnyResult<()> {
    for rename in renames.iter() {
        let new_name = rename
            .matched
            .vid_path
            .with_extension(&rename.matched.sub_ext_part);
        fs::rename(&rename.matched.sub_path, new_name)?;
    }
    Ok(())
}
