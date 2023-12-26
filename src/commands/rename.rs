use crate::commands::util;
use crate::commands::util::AskMatchAnswer;
use crate::config::{GlobalConfig, MatchFilesConfig};
use crate::scanner;
use crate::scanner::{MatchInfo, MatchInfoType, ScanOptions};
use anyhow::Result as AnyResult;
use rustyline::DefaultEditor;
use std::fs;

pub struct RenameCommand<'a> {
    global_conf: &'a GlobalConfig,
    conf: MatchFilesConfig,
    line_editor: Option<DefaultEditor>,
}

impl<'a> RenameCommand<'a> {
    pub fn new(global_conf: &'a GlobalConfig, rename_config: MatchFilesConfig) -> Self {
        RenameCommand {
            global_conf,
            conf: rename_config,
            line_editor: DefaultEditor::new().ok(),
        }
    }

    pub fn new_with_editor(
        global_conf: &'a GlobalConfig,
        rename_config: MatchFilesConfig,
        editor: Option<DefaultEditor>,
    ) -> Self {
        RenameCommand {
            global_conf,
            conf: rename_config,
            line_editor: editor,
        }
    }

    pub fn run(&mut self) -> AnyResult<()> {
        let matches = scanner::scan(ScanOptions::from_global_and_match_conf(
            self.global_conf,
            &self.conf,
        ))?;

        util::validate_sub_and_file_matches_ignore_extensions(self.global_conf, &matches)?;

        // Remove subs that already have the same name as their video file
        let renames: Vec<MatchInfo> = matches
            .into_iter()
            .filter(|match_info| match_info.match_type != MatchInfoType::Identical)
            .collect();

        if renames.is_empty() {
            println!("all subtitles are already renamed");
            return Ok(());
        }

        if !self.global_conf.confirm {
            rename_subtitles(&renames)?;
            return Ok(());
        }

        let match_ok_answer = util::ask_match_is_ok(
            &renames,
            self.conf.sub_area.as_ref(),
            self.conf.video_area.as_ref(),
            self.global_conf.color,
            self.line_editor.as_mut(),
        )?;

        match match_ok_answer {
            AskMatchAnswer::Yes => rename_subtitles(&renames)?,
            AskMatchAnswer::EditSubtitleRegex => loop {
                match util::get_user_regex(
                    "enter new subtitle area regex: ",
                    self.line_editor.as_mut(),
                ) {
                    Ok(Some(regex)) => {
                        let mut new_conf = self.conf.clone();
                        new_conf.sub_area = Some(regex);
                        if self.run_again(new_conf) {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => return Err(e),
                }
            },
            AskMatchAnswer::EditVideoRegex => loop {
                match util::get_user_regex(
                    "enter new video area regex: ",
                    self.line_editor.as_mut(),
                ) {
                    Ok(Some(regex)) => {
                        let mut new_conf = self.conf.clone();
                        new_conf.video_area = Some(regex);
                        if self.run_again(new_conf) {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => return Err(e),
                }
            },
            AskMatchAnswer::No => {}
        }
        Ok(())
    }

    fn run_again(&mut self, new_conf: MatchFilesConfig) -> bool {
        if let Err(e) =
            RenameCommand::new_with_editor(self.global_conf, new_conf, self.line_editor.take())
                .run()
        {
            println!("error: {}", e);
            false
        } else {
            true
        }
    }
}

fn rename_subtitles(renames: &[MatchInfo]) -> AnyResult<()> {
    for rename in renames.iter() {
        let new_name = rename.video_path.with_extension(&rename.sub_file_ext);
        fs::rename(&rename.sub_path, new_name)?;
    }
    Ok(())
}
