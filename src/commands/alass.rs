use crate::commands::util;
use crate::config::{AlassConfig, GlobalConfig};
use crate::scanner;
use crate::scanner::{ScanOptions, SubAndVid};
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
        let alass_binary = alass_binary()?;

        let matches = scanner::scan(ScanOptions {
            path: &self.global_conf.path,
            sub_area: self.conf.sub_area.as_ref(),
            video_area: self.conf.video_area.as_ref(),
            sub_filter: self.global_conf.sub_filter.as_ref(),
            video_filter: self.global_conf.video_filter.as_ref(),
        })?;

        util::validate_sub_and_file_matches(&self.global_conf, &matches)?;

        if self.global_conf.no_confirm || util::ask_user_ok(&matches, false)? {
            if self.conf.no_parallel {
                for m in matches {
                    self.align(&alass_binary, &m.matched)?;
                }
            } else {
                matches
                    .par_iter()
                    .try_for_each(|m| self.align(&alass_binary, &m.matched))?;
            }
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
