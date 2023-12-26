use crate::commands::util;
use crate::commands::util::AskMatchAnswer;
use crate::config::{AlassConfig, GlobalConfig};
use crate::scanner;
use crate::scanner::{MatchInfo, ScanOptions};
use anyhow::Result as AnyResult;
use rayon::prelude::*;
use rustyline::DefaultEditor;
use std::path::{Path, PathBuf};
use std::process::Command;

static ALASS_BINARY_NAMES: &[&str] = &["alass-cli", "alass"];

pub struct AlassCommand<'a> {
    global_conf: &'a GlobalConfig,
    conf: AlassConfig,
    line_editor: Option<DefaultEditor>,
}

impl<'a> AlassCommand<'a> {
    pub fn new(global_conf: &'a GlobalConfig, conf: AlassConfig) -> Self {
        Self {
            global_conf,
            conf,
            line_editor: DefaultEditor::new().ok(),
        }
    }

    pub fn new_with_editor(
        global_conf: &'a GlobalConfig,
        conf: AlassConfig,
        editor: Option<DefaultEditor>,
    ) -> Self {
        AlassCommand {
            global_conf,
            conf,
            line_editor: editor,
        }
    }

    pub fn run(&mut self) -> AnyResult<()> {
        let matches = scanner::scan(ScanOptions::from_global_and_match_conf(
            self.global_conf,
            &self.conf.match_config,
        ))?;

        util::validate_sub_and_file_matches(self.global_conf, &matches)?;

        if !self.global_conf.confirm {
            self.align_all(&matches)?;
            return Ok(());
        }

        let match_ok_answer = util::ask_match_is_ok(
            &matches,
            self.conf.match_config.sub_area.as_ref(),
            self.conf.match_config.video_area.as_ref(),
            self.global_conf.color,
            self.line_editor.as_mut(),
        )?;

        match match_ok_answer {
            AskMatchAnswer::Yes => self.align_all(&matches)?,
            AskMatchAnswer::EditSubtitleRegex => loop {
                match util::get_user_regex(
                    "enter new subtitle area regex: ",
                    self.line_editor.as_mut(),
                ) {
                    Ok(Some(regex)) => {
                        let mut new_conf = self.conf.clone();
                        new_conf.match_config.sub_area = Some(regex);
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
                        new_conf.match_config.video_area = Some(regex);
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

    fn run_again(&mut self, new_conf: AlassConfig) -> bool {
        if let Err(e) =
            AlassCommand::new_with_editor(self.global_conf, new_conf, self.line_editor.take()).run()
        {
            println!("error: {}", e);
            false
        } else {
            true
        }
    }

    fn align_all(&self, aligns: &[MatchInfo]) -> AnyResult<()> {
        let alass_binary = alass_binary()?;

        if self.conf.no_parallel {
            for m in aligns {
                self.align(&alass_binary, &m.sub_path, &m.video_path)?;
            }
        } else {
            aligns
                .par_iter()
                .try_for_each(|m| self.align(&alass_binary, &m.sub_path, &m.video_path))?;
        }
        Ok(())
    }

    fn align(&self, alass_binary: &Path, sub_path: &Path, video_path: &Path) -> AnyResult<()> {
        let mut cmd = Command::new(alass_binary);

        cmd.arg(video_path).arg(sub_path).arg(sub_path);

        if !self.conf.flags.is_empty() {
            cmd.args(&self.conf.flags);
        }

        let exit_code = cmd.current_dir(&self.global_conf.path).spawn()?.wait()?;

        if !exit_code.success() {
            bail!("sub-batch: `{:?}` failed", alass_binary);
        }
        Ok(())
    }
}

fn alass_binary() -> AnyResult<PathBuf> {
    for bin in ALASS_BINARY_NAMES {
        if let Ok(bin) = which::which(bin) {
            return Ok(bin);
        }
    }
    bail!(
        "could not find any of the following in PATH: {}\n\
        See https://github.com/kaegi/alass for install instructions.",
        ALASS_BINARY_NAMES.join(", ")
    )
}
