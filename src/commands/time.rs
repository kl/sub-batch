use crate::commands::util;
use crate::config::{GlobalConfig, TimeConfig};
use crate::scanner::{self, AreaScan, ScanOptions};
use anyhow::Result as AnyResult;
use std::fs;
use subparse::timetypes::TimeDelta;
use subparse::SubtitleFile;

pub fn run(global_conf: &GlobalConfig, conf: TimeConfig) -> AnyResult<()> {
    let matches = scanner::scan_subs_only(ScanOptions::from_global_conf(
        global_conf,
        None,
        AreaScan::Normal,
        None,
        AreaScan::Normal,
    ))?;
    util::validate_sub_matches(global_conf, &matches)?;

    let mut parsed_subs: Vec<SubtitleFile> = matches
        .iter()
        .map(|path| -> AnyResult<SubtitleFile> {
            let content = fs::read(path)?;
            let format = subparse::get_subtitle_format(path.extension(), &content)
                .ok_or_else(|| anyhow!("invalid subtitle format: {:?}", path.extension()))?;

            subparse::parse_bytes(format, &content, Some(conf.encoding), conf.fps)
                .map_err(|e| anyhow!("failed to parse subtitle file: {:?}", e))
        })
        .collect::<AnyResult<_>>()?;

    for (i, sub) in parsed_subs.iter_mut().enumerate() {
        let mut entries = sub
            .get_subtitle_entries()
            .map_err(|e| anyhow!("failed to get subtitle entries: {:?}", e))?;

        for entry in &mut entries {
            entry.timespan += TimeDelta::from_msecs(conf.timing);
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
