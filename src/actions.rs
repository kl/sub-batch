use crate::config::Config;
use crate::sub_transformer::{Action, SubAndFile};
use crate::util::AnyError;
use crate::util::FoldResultsVecExt;
use std::fs;
use subparse::timetypes::TimeDelta;

#[derive(Debug)]
pub struct Renamer {}

impl Renamer {
    pub fn new() -> Renamer {
        Renamer {}
    }
}

impl Action for Renamer {
    fn apply(&self, matches: &[SubAndFile], config: &Config) -> Result<(), AnyError> {
        if !config.rename {
            return Ok(());
        }

        for rename in matches.iter() {
            println!("{} -> {}", rename.file_path, rename.sub_path);
        }

        println!("Ok? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.to_lowercase().starts_with('y') {
            for rename in matches.iter() {
                let sub_ext = rename.sub_ext();

                let new_name = if let Some(file_ext) = rename.file_ext() {
                    rename.file_path.replace(file_ext, sub_ext)
                } else {
                    rename.file_path.to_string() + sub_ext
                };

                fs::rename(rename.sub_path, new_name)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct TimingAdjuster {}

impl TimingAdjuster {
    pub fn new() -> TimingAdjuster {
        TimingAdjuster {}
    }
}

impl Action for TimingAdjuster {
    fn apply(&self, matches: &[SubAndFile], config: &Config) -> Result<(), AnyError> {
        if let Some(timing) = config.timing {
            let mut parsed_subs = matches
                .iter()
                .map(|s| -> Result<Box<_>, AnyError> {
                    let content = fs::read(s.sub_path)?;
                    let format = subparse::get_subtitle_format_err(s.sub_ext(), &content)?;
                    let parsed =
                        subparse::parse_bytes(format, &content, config.encoding, config.fps)?;
                    Ok(parsed)
                })
                .fold_results_vec()?;

            for (i, sub) in parsed_subs.iter_mut().enumerate() {
                let mut entries = sub.get_subtitle_entries()?;
                for entry in &mut entries {
                    entry.timespan += TimeDelta::from_msecs(timing);
                }
                sub.update_subtitle_entries(&entries)?;

                fs::write(matches[i].sub_path, sub.to_data()?)?;
            }
        }

        Ok(())
    }
}
