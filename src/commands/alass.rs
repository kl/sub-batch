use crate::commands::util;
use crate::commands::util::AskMatchAnswer;
use crate::config::{AlassConfig, GlobalConfig};
use crate::scanner;
use crate::scanner::{MatchInfo, ScanOptions, SubAndVid};
use anyhow::Result as AnyResult;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

static ALASS_BINARY_NAMES: &[&str] = &["alass-cli", "alass"];

pub struct AlassCommand {
    global_conf: GlobalConfig,
    conf: AlassConfig,
}

impl AlassCommand {
    pub fn new(global_conf: GlobalConfig, conf: AlassConfig) -> Self {
        Self { global_conf, conf }
    }

    pub fn run(&self) -> AnyResult<()> {
        let matches = scanner::scan(ScanOptions {
            path: &self.global_conf.path,
            sub_area: self.conf.sub_area.as_ref(),
            video_area: self.conf.video_area.as_ref(),
            sub_filter: self.global_conf.sub_filter.as_ref(),
            video_filter: self.global_conf.video_filter.as_ref(),
        })?;

        util::validate_sub_and_file_matches(&self.global_conf, &matches)?;

        if self.global_conf.no_confirm {
            self.align_all(&matches)?;
            return Ok(());
        }

        match util::ask_match_is_ok(
            &matches,
            self.conf.sub_area.as_ref(),
            self.conf.video_area.as_ref(),
            true,
        )? {
            AskMatchAnswer::Yes => self.align_all(&matches)?,
            AskMatchAnswer::EditSubtitleRegex => loop {
                let mut new_conf = self.conf.clone();
                new_conf.sub_area = Some(util::get_user_regex("input new subtitle area regex: ")?);
                if let Err(e) = AlassCommand::new(self.global_conf.clone(), new_conf).run() {
                    println!("error: {}", e);
                } else {
                    break;
                }
            },
            AskMatchAnswer::EditVideoRegex => loop {
                let mut new_conf = self.conf.clone();
                new_conf.video_area = Some(util::get_user_regex("input new video area regex: ")?);
                if let Err(e) = AlassCommand::new(self.global_conf.clone(), new_conf).run() {
                    println!("error: {}", e);
                } else {
                    break;
                }
            },
            AskMatchAnswer::No => {}
        }
        Ok(())
    }

    fn align_all(&self, aligns: &[MatchInfo]) -> AnyResult<()> {
        let alass_binary = alass_binary()?;

        if self.conf.no_parallel {
            for m in aligns {
                self.align(&alass_binary, &m.matched)?;
            }
        } else {
            aligns
                .par_iter()
                .try_for_each(|m| self.align(&alass_binary, &m.matched))?;
        }
        Ok(())
    }

    fn align(&self, alass_binary: &Path, target: &SubAndVid) -> AnyResult<()> {
        let mut cmd = Command::new(alass_binary);

        cmd.arg(&target.vid_path)
            .arg(&target.sub_path)
            .arg(&target.sub_path);

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
