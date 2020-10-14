use crate::config::Config;
use crate::sub_transformer::{Action, SubAndFile};
use anyhow::Result as AnyResult;
use std::fs;
use subparse::timetypes::TimeDelta;
use subparse::SubtitleFile;

#[derive(Debug)]
pub struct Renamer {}

impl Renamer {
    pub fn new() -> Renamer {
        Renamer {}
    }
}

impl Action for Renamer {
    fn apply(&self, matches: &[SubAndFile], config: &Config) -> AnyResult<()> {
        if !config.rename {
            return Ok(());
        }

        let renames: Vec<&SubAndFile> = matches
            .iter()
            .filter(|re| re.sub_file_part != re.file_file_part)
            .collect();

        if renames.is_empty() {
            return Ok(());
        }

        if config.no_confirm || ask_user_ok(&renames)? {
            for rename in renames.iter() {
                let new_name = rename.file_path.with_extension(rename.sub_ext_part);
                fs::rename(rename.sub_path, new_name)?;
            }
        }

        Ok(())
    }
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

#[derive(Debug)]
pub struct TimingAdjuster {}

impl TimingAdjuster {
    pub fn new() -> TimingAdjuster {
        TimingAdjuster {}
    }
}

impl Action for TimingAdjuster {
    fn apply(&self, matches: &[SubAndFile], config: &Config) -> AnyResult<()> {
        if let Some(timing) = config.timing {
            let mut parsed_subs: Vec<SubtitleFile> = matches
                .iter()
                .map(|s| -> AnyResult<SubtitleFile> {
                    let content = fs::read(s.sub_path)?;
                    let format = subparse::get_subtitle_format(Some(s.sub_ext_part), &content)
                        .ok_or_else(|| anyhow!("invalid subtitle format: {:?}", s.sub_ext_part))?;

                    subparse::parse_bytes(format, &content, config.encoding, config.fps)
                        .map_err(|e| anyhow!("failed to parse subtitle file: {:?}", e))
                })
                .collect::<AnyResult<_>>()?;

            for (i, sub) in parsed_subs.iter_mut().enumerate() {
                let mut entries = sub
                    .get_subtitle_entries()
                    .map_err(|e| anyhow!("failed to get subtitle entries: {:?}", e))?;

                for entry in &mut entries {
                    entry.timespan += TimeDelta::from_msecs(timing);
                }
                sub.update_subtitle_entries(&entries)
                    .map_err(|e| anyhow!("failed to update subtitle entries: {:?}", e))?;

                let data = sub
                    .to_data()
                    .map_err(|e| anyhow!("failed to get subtitle data: {:?}", e))?;

                fs::write(matches[i].sub_path, data)?;
            }
        }

        Ok(())
    }
}
