use crate::config::{GlobalConfig, RenameConfig};
use crate::scanner;
use crate::scanner::{ScanOptions, SubAndFile};
use anyhow::Result as AnyResult;
use std::fs;

pub fn run(global_conf: GlobalConfig, conf: RenameConfig) -> AnyResult<()> {
    let matches = scanner::scan(ScanOptions::new(
        global_conf.path,
        conf.sub_area,
        conf.video_area,
    ))?;

    let renames: Vec<&SubAndFile> = matches
        .iter()
        .filter(|re| re.sub_file_part != re.file_file_part)
        .collect();

    if renames.is_empty() {
        return Ok(());
    }

    if global_conf.no_confirm || ask_user_ok(&renames)? {
        for rename in renames.iter() {
            let new_name = rename.file_path.with_extension(&rename.sub_ext_part);
            fs::rename(&rename.sub_path, new_name)?;
        }
    }

    Ok(())
}

fn ask_user_ok(renames: &[&SubAndFile]) -> AnyResult<bool> {
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
