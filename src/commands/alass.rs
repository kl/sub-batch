use crate::commands::util;
use crate::config::{AlassConfig, GlobalConfig};
use crate::scanner;
use crate::scanner::{ScanOptions, SubAndFile};
use anyhow::Result as AnyResult;
use rayon::prelude::*;
use std::path::Path;
use std::process::Command;

static ALASS_EXE: &str = "alass-cli";

pub fn run(global_conf: GlobalConfig, conf: AlassConfig) -> AnyResult<()> {
    check_alass_installed()?;

    let matches = scanner::scan(ScanOptions::new(
        global_conf.path.clone(),
        conf.sub_area.clone(),
        conf.video_area.clone(),
    ))?;

    if matches.is_empty() {
        return Ok(());
    }

    if global_conf.no_confirm || util::ask_user_ok(&matches)? {
        if conf.no_parallel {
            for m in matches {
                align(&global_conf.path, &conf.flags, &m)?;
            }
        } else {
            matches
                .par_iter()
                .try_for_each(|m| align(&global_conf.path, &conf.flags, &m))?;
        }
    }

    Ok(())
}

fn check_alass_installed() -> AnyResult<()> {
    which::which(ALASS_EXE).map(|_| ()).map_err(|_| {
        anyhow!(
            "could not find `{}` in PATH. \
             See https://github.com/kaegi/alass for install instructions.",
            ALASS_EXE
        )
    })
}

fn align(dir: &Path, flags: &[String], target: &SubAndFile) -> AnyResult<()> {
    let mut cmd = Command::new(ALASS_EXE);

    let exit_code = if flags.is_empty() {
        cmd.current_dir(dir)
            .arg(&target.file_path)
            .arg(&target.sub_path)
            .arg(&target.sub_path)
            .spawn()?
            .wait()?
    } else {
        cmd.current_dir(dir)
            .arg(&target.file_path)
            .arg(&target.sub_path)
            .arg(&target.sub_path)
            .args(flags)
            .spawn()?
            .wait()?
    };

    if !exit_code.success() {
        bail!("alass-cli error");
    }

    Ok(())
}
