use crate::config::GlobalConfig;
use anyhow::Result as AnyResult;
use regex::{Match, Regex};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref NUMBER: Regex = Regex::new(r"\d+").unwrap();
}

static EXTENSIONS: &[&str] = &[
    "cdg", "idx", "srt", "sub", "utf", "ass", "ssa", "aqt", "jss", "psb", "rt", "sami", "smi",
    "smil", "stl", "usf", "dks", "pjs", "mpl2", "mks", "vtt", "tt", "ttml", "dfxp", "scc", "itt",
    "sbv", "aaf", "mcc", "mxf", "asc", "cap", "onl", "cin", "ult", "scr", "sst", "nav", "son",
];

#[derive(Debug)]
pub struct SubAndVid {
    pub sub_path: PathBuf,
    pub sub_file_part: String, // lossy if not UTF-8
    pub sub_ext_part: OsString,
    pub vid_path: PathBuf,
    pub vid_file_part: String, // lossy if not UTF-8
    pub vid_ext_part: Option<OsString>,
}

#[derive(Debug)]
pub struct MatchInfo {
    pub matched: SubAndVid,
    match_type: MatchType,
}

#[derive(Debug, PartialEq)]
enum MatchType {
    // If the sub file (minus extension) is identical to the video file (minus extension)
    Identical,
    // These ranges are indices in to the sub and file file path strings in the SubAndVid struct
    Range {
        sub_match: std::ops::Range<usize>,
        vid_match: std::ops::Range<usize>,
    },
}

impl MatchInfo {
    pub fn sub_match_parts(&self) -> (&str, &str, &str) {
        match &self.match_type {
            MatchType::Identical => (&self.matched.sub_file_part, "", ""),
            MatchType::Range { sub_match, .. } => {
                let before = &self.matched.sub_file_part[0..sub_match.start];
                let matched = &self.matched.sub_file_part[sub_match.clone()];
                let after = &self.matched.sub_file_part[sub_match.end..];
                (before, matched, after)
            }
        }
    }

    pub fn vid_match_parts(&self) -> (&str, &str, &str) {
        match &self.match_type {
            MatchType::Identical => (&self.matched.vid_file_part, "", ""),
            MatchType::Range { vid_match, .. } => {
                let before = &self.matched.vid_file_part[0..vid_match.start];
                let matched = &self.matched.vid_file_part[vid_match.clone()];
                let after = &self.matched.vid_file_part[vid_match.end..];
                (before, matched, after)
            }
        }
    }
}

impl SubAndVid {
    fn new(sub_path: impl Into<PathBuf>, vid_path: impl Into<PathBuf>) -> SubAndVid {
        let sub_path = sub_path.into();
        let vid_path = vid_path.into();

        let (sub_file_part, sub_ext_part) = split_extension(&sub_path).unwrap();

        let (vid_file_part, vid_ext_part) = split_extension(&vid_path)
            .map(|(f, e)| (f, Some(e)))
            .unwrap_or((vid_path.as_os_str(), None));

        let sub_file_part = sub_file_part.to_owned().to_string_lossy().to_string();
        let sub_ext_part = sub_ext_part.to_owned();

        let vid_file_part = vid_file_part.to_owned().to_string_lossy().to_string();

        SubAndVid {
            sub_path,
            sub_file_part,
            sub_ext_part,
            vid_path,
            vid_file_part,
            vid_ext_part,
        }
    }
}

#[derive(Debug)]
pub struct ScanOptions<'a> {
    pub path: &'a Path,
    pub sub_area: Option<&'a Regex>,
    pub video_area: Option<&'a Regex>,
    pub sub_filter: Option<&'a Regex>,
    pub video_filter: Option<&'a Regex>,
    pub sub_area_scan: AreaScan,
    pub video_area_scan: AreaScan,
}

#[derive(Debug, Copy, Clone)]
pub enum AreaScan {
    Normal,
    Reverse,
}

impl<'a> ScanOptions<'a> {
    pub fn from_global_conf(
        conf: &'a GlobalConfig,
        sub_area: Option<&'a Regex>,
        sub_area_scan: AreaScan,
        video_area: Option<&'a Regex>,
        video_area_scan: AreaScan,
    ) -> Self {
        ScanOptions {
            path: &conf.path,
            sub_area,
            video_area,
            sub_filter: conf.sub_filter.as_ref(),
            sub_area_scan,
            video_filter: conf.video_filter.as_ref(),
            video_area_scan,
        }
    }
}

pub fn scan(options: ScanOptions) -> AnyResult<Vec<MatchInfo>> {
    let files_with_numbers = scan_number_files(&options)?;
    let matched = match_files(&options, &files_with_numbers)?;
    Ok(matched)
}

fn scan_number_files(options: &ScanOptions) -> AnyResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = entries(options.path)?
        .iter()
        .map(|e| e.path())
        .filter(|p| p.is_file() && NUMBER.is_match(&p.to_string_lossy()))
        .collect();

    files.sort();

    Ok(files)
}

pub fn scan_subs_only(options: ScanOptions) -> AnyResult<Vec<PathBuf>> {
    let subs = entries(options.path)?
        .into_iter()
        .map(|e| e.path())
        .filter(|p| {
            let ext = p.extension().and_then(OsStr::to_str).unwrap_or_default();
            p.is_file() && EXTENSIONS.contains(&ext)
        })
        .filter(|sub| regex_matches_file_name(options.sub_filter, sub))
        .collect();

    Ok(subs)
}

fn entries(path: &Path) -> io::Result<Vec<DirEntry>> {
    std::fs::read_dir(path)?.collect::<io::Result<Vec<DirEntry>>>()
}

fn match_files(options: &ScanOptions, files_with_numbers: &[PathBuf]) -> AnyResult<Vec<MatchInfo>> {
    // Separate subtitle files from non-subtitle files.
    let (subs, others): (Vec<&PathBuf>, Vec<&PathBuf>) =
        files_with_numbers.iter().partition(|file| {
            let ext = file.extension().and_then(OsStr::to_str).unwrap_or_default();
            EXTENSIONS.contains(&ext)
        });

    // Remove files that don't match the filters.
    let subs = subs
        .into_iter()
        .filter(|sub| regex_matches_file_name(options.sub_filter, sub))
        .collect();
    let others = others
        .into_iter()
        .filter(|other| regex_matches_file_name(options.video_filter, other))
        .collect();

    // Find the areas inside the paths that match the area regular expressions.
    let sub_areas = find_areas(subs, &options.sub_area)?;
    let other_areas = find_areas(others, &options.video_area)?;

    // Match the subtitle and other paths where they have the same number in their areas.
    match_areas(sub_areas, other_areas, options)
}

fn match_areas(
    sub_areas: Vec<PathAndArea>,
    other_areas: Vec<PathAndArea>,
    options: &ScanOptions,
) -> AnyResult<Vec<MatchInfo>> {
    // Partition the subtitles so that subs with the same file stem (but different extensions)
    // are in the same vec, e.g. sub1.srt, sub1.en.srt, sub1.jp.srt are put in the same vec.
    let partition_map: HashMap<&OsStr, Vec<PathAndArea>> =
        sub_areas.into_iter().fold(HashMap::new(), |mut map, sub| {
            let (stem, _) = split_extension(sub.path).unwrap();
            map.entry(stem).or_default().push(sub);
            map
        });
    let mut sub_partitions = partition_map.iter().collect::<Vec<_>>();
    sub_partitions.sort_unstable_by_key(|subs| subs.0);

    let mut other_map: HashMap<&OsStr, PathAndArea> =
        other_areas
            .into_iter()
            .fold(HashMap::new(), |mut map, other| {
                let stem =
                // We may not have an extension, if not return the entire path
                split_extension(other.path)
                    .map(|(other_file_part, _)| other_file_part)
                    .unwrap_or(other.path.file_stem().unwrap_or(OsStr::new("")));
                map.insert(stem, other);
                map
            });

    let mut already_matched: Vec<MatchInfo> = Vec::new();
    sub_partitions.retain(|(stem, subs)| {
        if let Some(other) = other_map.remove(*stem) {
            for sub in subs.iter() {
                already_matched.push(MatchInfo {
                    matched: SubAndVid::new(sub.path, other.path),
                    match_type: MatchType::Identical,
                })
            }
            false
        } else {
            true
        }
    });

    // Put others back into a Vec and sort to make sure we have a deterministic order
    let mut other_areas = other_map.into_values().collect::<Vec<_>>();
    other_areas.sort_unstable_by_key(|other| other.path);

    let mut matched: Vec<MatchInfo> = sub_partitions
        .iter()
        .filter_map(|(_, subs)| {
            let first = &subs[0];

            let num_range =
                find_sub_number_in_area(&first.area, options.sub_area_scan).map(|m| m.range())?;
            let num = first.area[num_range.clone()]
                .parse::<u32>()
                .unwrap()
                .to_string(); // remove leading zeroes

            let (other_index, other, position) = other_areas
                .iter()
                .enumerate()
                .find(|(_, other)| other.area.contains(&num))
                .map(|(index, other)| {
                    (
                        index,
                        other,
                        find_specific_number_in_area(&other.area, &num, options.video_area_scan)
                            .unwrap(),
                    )
                })?;

            // If the first sub in the set matched, all other subs in the set should also be matched
            let matched = subs
                .iter()
                .map({
                    |sub| MatchInfo {
                        matched: SubAndVid::new(sub.path, other.path),
                        match_type: MatchType::Range {
                            sub_match: (sub.area_start_index + num_range.start)
                                ..(sub.area_start_index + num_range.end),

                            vid_match: (other.area_start_index + position)
                                ..(other.area_start_index + position + num.len()),
                        },
                    }
                })
                .collect::<Vec<MatchInfo>>();

            other_areas.remove(other_index);
            Some(matched)
        })
        .flatten()
        .collect();

    matched.extend(already_matched);
    Ok(matched)
}

fn find_sub_number_in_area(area: &str, area_scan: AreaScan) -> Option<Match> {
    let mut matches = NUMBER.find_iter(area);
    match area_scan {
        AreaScan::Normal => matches.next(),
        AreaScan::Reverse => matches.last(),
    }
}

fn find_specific_number_in_area(area: &str, number: &str, area_scan: AreaScan) -> Option<usize> {
    match area_scan {
        AreaScan::Normal => area.find(number),
        AreaScan::Reverse => area.rfind(number),
    }
}

fn regex_matches_file_name(regex: Option<&Regex>, path: &Path) -> bool {
    match regex {
        Some(regex) => match try_extract_file_name(path) {
            Ok(file_name) => regex.is_match(&file_name),
            _ => false,
        },
        None => true,
    }
}

fn split_extension(path: &Path) -> Option<(&OsStr, OsString)> {
    let stem = path.file_stem()?;
    let ext = path.extension()?;

    let stem2 = Path::new(stem).file_stem()?;

    if stem2 == stem {
        // Single extension file (e.g. subtitle.srt)
        Some((stem, ext.to_os_string()))
    } else {
        // Double extension file (e.g. subtitle.en.srt)
        let ext2 = Path::new(stem).extension()?;

        // If the secondary extension includes a number (e.g. subtitle.2.srt) we ignore it
        // because it may contain the number part that matches the video file, and if it
        // is more than 3 characters long we also ignore it (because mpv will not auto detect
        // subtitle files when the secondary extension is longer than 3 characters).
        if NUMBER.is_match(&ext2.to_string_lossy()) || ext2.to_string_lossy().chars().count() > 3 {
            Some((stem, ext.to_os_string()))
        } else {
            let mut extensions = OsString::from(ext2);
            extensions.push(".");
            extensions.push(ext);
            Some((stem2, extensions))
        }
    }
}

fn find_areas<'a>(
    paths: Vec<&'a PathBuf>,
    area_matcher: &Option<&Regex>,
) -> AnyResult<Vec<PathAndArea<'a>>> {
    paths
        .iter()
        .map(|path| -> AnyResult<PathAndArea> {
            let (area, area_start_index) = try_extract_area(path, area_matcher)?;
            Ok(PathAndArea {
                path,
                area,
                area_start_index,
            })
        })
        .collect::<AnyResult<_>>()
}

fn try_extract_area(path: &Path, regex: &Option<&Regex>) -> AnyResult<(String, usize)> {
    let name = try_extract_file_name(path)?;

    if let Some(r) = regex {
        if let Some(m) = r.find(&name) {
            Ok((m.as_str().into(), m.start()))
        } else {
            bail!("failed to match regex {} on text: {}", r, name);
        }
    } else {
        Ok((name, 0))
    }
}

fn try_extract_file_name(path: &Path) -> AnyResult<String> {
    Ok(path
        .file_name()
        .ok_or_else(|| anyhow!("file {} has an invalid file name", path.to_string_lossy()))?
        .to_string_lossy()
        .to_string())
}

#[derive(Debug)]
struct PathAndArea<'a> {
    path: &'a Path,
    area: String,
    // Where the area starts in path
    area_start_index: usize,
}
