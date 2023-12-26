use crate::commands::util;
use crate::config::{GlobalConfig, TimeConfig};
use crate::scanner::{self, AreaScan, ScanOptions, SecondaryExtensionPolicy};
use anyhow::Result as AnyResult;
use std::fs;
use subparse::timetypes::TimeDelta;
use subparse::SubtitleFile;

pub struct TimeCommand<'a> {
    global_conf: &'a GlobalConfig,
    conf: TimeConfig,
}

impl<'a> TimeCommand<'a> {
    pub fn new(global_conf: &'a GlobalConfig, conf: TimeConfig) -> Self {
        TimeCommand { global_conf, conf }
    }

    pub fn run(&self) -> AnyResult<()> {
        let matches = scanner::scan_subs_only(ScanOptions::from_global_conf(
            self.global_conf,
            None,
            AreaScan::Normal,
            None,
            AreaScan::Normal,
            SecondaryExtensionPolicy::Never,
        ))?;
        util::validate_sub_matches(self.global_conf, &matches)?;

        let mut parsed_subs: Vec<SubtitleFile> = matches
            .iter()
            .map(|path| -> AnyResult<SubtitleFile> {
                let content = fs::read(path)?;
                let format = subparse::get_subtitle_format(path.extension(), &content)
                    .ok_or_else(|| anyhow!("invalid subtitle format: {:?}", path.extension()))?;

                subparse::parse_bytes(format, &content, Some(self.conf.encoding), self.conf.fps)
                    .map_err(|e| anyhow!("failed to parse subtitle file: {:?}", e))
            })
            .collect::<AnyResult<_>>()?;

        for (i, sub) in parsed_subs.iter_mut().enumerate() {
            let mut entries = sub
                .get_subtitle_entries()
                .map_err(|e| anyhow!("failed to get subtitle entries: {:?}", e))?;

            for entry in &mut entries {
                entry.timespan += TimeDelta::from_msecs(self.conf.timing);
            }
            sub.update_subtitle_entries(&entries)
                .map_err(|e| anyhow!("failed to update subtitle entries: {:?}", e))?;

            let data = sub
                .to_data()
                .map_err(|e| anyhow!("failed to get subtitle data: {:?}", e))?;

            fs::write(&matches[i], data)?;
        }
        Ok(())
    }
}
