use crate::config::GlobalConfig;
use anyhow::Result as AnyResult;
use regex::{Match, Regex};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::fs::DirEntry;
use std::io;
use std::ops::Range;
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
pub struct MatchInfo {
    pub sub_path: PathBuf,
    pub video_path: PathBuf,

    pub sub_file_ext: OsString,

    /// lossy if not valid Unicode
    pub sub_file_name: String,

    /// lossy if not valid Unicode
    pub video_file_name: String,

    pub match_type: MatchInfoType,
}

#[derive(Debug, PartialEq)]
pub enum MatchInfoType {
    NumberMatch {
        /// index range into sub_file_name
        sub_number_range: Range<usize>,

        /// index range into video_file_name
        video_number_range: Range<usize>,

        /// index range into sub_file_name
        sub_match_area: Option<Range<usize>>,

        /// index range into video_file_name
        video_match_area: Option<Range<usize>>,
    },
    Identical,
}

impl MatchInfo {
    fn identical(sub: &FileInfo, video: &FileInfo) -> Self {
        MatchInfo {
            sub_path: sub.path.to_path_buf(),
            video_path: video.path.to_path_buf(),
            sub_file_name: sub.file_name.clone(),
            sub_file_ext: sub.ext.clone().unwrap(),
            video_file_name: video.file_name.clone(),
            match_type: MatchInfoType::Identical,
        }
    }

    fn from_number_ranges(
        sub: &FileInfo,
        video: &FileInfo,
        sub_number_range: Range<usize>,
        video_number_range: Range<usize>,
    ) -> Self {
        let mut base = MatchInfo::identical(sub, video);
        base.match_type = MatchInfoType::NumberMatch {
            sub_number_range,
            video_number_range,
            sub_match_area: sub.area_range.clone(),
            video_match_area: video.area_range.clone(),
        };
        base
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
    pub secondary_ext_policy: SecondaryExtensionPolicy,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AreaScan {
    Normal,
    Reverse,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SecondaryExtensionPolicy {
    Always,
    Never,
    Maybe,
}

impl<'a> ScanOptions<'a> {
    pub fn from_global_conf(
        conf: &'a GlobalConfig,
        sub_area: Option<&'a Regex>,
        sub_area_scan: AreaScan,
        video_area: Option<&'a Regex>,
        video_area_scan: AreaScan,
        secondary_ext_policy: SecondaryExtensionPolicy,
    ) -> Self {
        ScanOptions {
            path: &conf.path,
            sub_area,
            video_area,
            sub_filter: conf.sub_filter.as_ref(),
            sub_area_scan,
            video_filter: conf.video_filter.as_ref(),
            video_area_scan,
            secondary_ext_policy,
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
    let sub_infos = parse_file_info(subs, &options.sub_area, options.secondary_ext_policy)?;
    let other_infos =
        parse_file_info(others, &options.video_area, SecondaryExtensionPolicy::Never)?;

    // Match the subtitle and other paths where they have the same number in their areas.
    match_areas(sub_infos, other_infos, options)
}

fn match_areas(
    subs: Vec<FileInfo>,
    others: Vec<FileInfo>,
    options: &ScanOptions,
) -> AnyResult<Vec<MatchInfo>> {
    let mut sub_stems = sub_stem_map(subs)
        .into_iter()
        .collect::<Vec<(&OsStr, Vec<FileInfo>)>>();
    sub_stems.sort_unstable_by_key(|subs| subs.0);

    let mut other_stems = other_stem_map(others);

    let mut already_matched: Vec<MatchInfo> = Vec::new();

    sub_stems.retain(|(stem, subs)| {
        // Can we match a sub stem to a video stem perfectly? If so they are already matched
        // so remove the sub(s) and other.
        if let Some(other) = other_stems.remove(*stem) {
            for sub in subs.iter() {
                already_matched.push(MatchInfo::identical(sub, &other));
            }
            false
        } else {
            true
        }
    });

    // Put others back into a Vec and sort to make sure we have a deterministic order
    let mut other_partitions = other_stems.into_values().collect::<Vec<_>>();
    other_partitions.sort_unstable_by_key(|other| other.path);

    // Match subs and video files based on the numbers in their respective embedded areas.
    let mut matched: Vec<MatchInfo> = sub_stems
        .iter()
        .filter_map(|(_, subs)| {
            let first = &subs[0];
            let (num, num_range) = first.find_number_in_area(options.sub_area_scan)?;

            let (other_index, other, other_num_range) = other_partitions
                .iter()
                .enumerate()
                .filter_map(|(index, other)| {
                    let num_range =
                        other.find_specific_number_in_area(num, options.video_area_scan)?;
                    Some((index, other, num_range))
                })
                .next()?;

            // If the first sub in the set matched, all other subs in the set should also be matched
            let matched = subs
                .iter()
                .map({
                    |sub| {
                        MatchInfo::from_number_ranges(
                            sub,
                            other,
                            num_range.clone(),
                            other_num_range.clone(),
                        )
                    }
                })
                .collect::<Vec<MatchInfo>>();

            other_partitions.remove(other_index);
            Some(matched)
        })
        .flatten()
        .collect();

    matched.extend(already_matched);
    Ok(matched)
}

// Partition the files so that files with the same file stem (but different extensions)
// are in the same vec, e.g. sub1.srt, sub1.en.srt, sub1.jp.srt are put in the same vec.
fn sub_stem_map(files: Vec<FileInfo>) -> HashMap<&OsStr, Vec<FileInfo>> {
    files.into_iter().fold(HashMap::new(), |mut map, file| {
        map.entry(file.stem).or_default().push(file);
        map
    })
}

// Video files do not include secondary extensions (so e.g. vid.en.mp4 and vid.jp.mp4 are
// always treated as two distinct files).
fn other_stem_map(files: Vec<FileInfo>) -> HashMap<&OsStr, FileInfo> {
    files.into_iter().fold(HashMap::new(), |mut map, file| {
        map.insert(file.stem, file);
        map
    })
}

fn regex_matches_file_name(regex: Option<&Regex>, path: &Path) -> bool {
    match regex {
        Some(regex) => match path.file_name() {
            Some(file_name) => regex.is_match(&file_name.to_string_lossy()),
            _ => false,
        },
        None => true,
    }
}

#[derive(Debug)]
struct FileInfo<'a> {
    path: &'a Path,
    stem: &'a OsStr,
    // Where the area is in file_name (if we have an area)
    area_range: Option<Range<usize>>,
    // The (possibly double) file extension
    ext: Option<OsString>,
    // Where the (possibly double) extension is in file_name (if an extension exists)
    ext_start_index: Option<usize>,
    // lossy if not valid Unicode
    file_name: String,
}

impl FileInfo<'_> {
    fn find_number_in_area(&self, area_scan: AreaScan) -> Option<(&str, Range<usize>)> {
        self.find_in_area(&NUMBER, area_scan)
    }

    fn find_specific_number_in_area(
        &self,
        number: &str,
        area_scan: AreaScan,
    ) -> Option<Range<usize>> {
        let regex = Regex::new(number).expect("invalid number");
        self.find_in_area(&regex, area_scan).map(|(_, range)| range)
    }

    fn find_in_area(&self, regex: &Regex, area_scan: AreaScan) -> Option<(&str, Range<usize>)> {
        let (area, area_start) = self.area_and_start();

        let mut matches = regex.find_iter(area).collect::<Vec<Match>>();
        if area_scan == AreaScan::Reverse {
            matches.reverse();
        }

        for num_match in matches {
            let num = remove_leading_zeroes(num_match.as_str());
            let removed_zeroes = num_match.as_str().len() - num.len();
            let range =
                (area_start + num_match.start() + removed_zeroes)..(area_start + num_match.end());

            let result = (num_match.as_str(), range);

            // We must check and make sure that the matched number isn't part of the file extension. If it is we
            // try the next matching number or return None when there are no more matching numbers.
            match self.ext_start_index {
                Some(ext_start) if result.1.end < ext_start => return Some(result),
                None => return Some(result),
                _ => {}
            }
        }
        None
    }

    fn area_and_start(&self) -> (&str, usize) {
        if let Some(area_range) = self.area_range.as_ref() {
            (
                &self.file_name[area_range.start..area_range.end],
                area_range.start,
            )
        } else {
            (&self.file_name, 0)
        }
    }
}

fn remove_leading_zeroes(num: &str) -> &str {
    if let Some(non_zero) = num.chars().position(|n| n != '0') {
        &num[non_zero..]
    } else {
        "0"
    }
}

fn parse_file_info<'a>(
    paths: Vec<&'a PathBuf>,
    area_matcher: &Option<&Regex>,
    secondary_ext_policy: SecondaryExtensionPolicy,
) -> AnyResult<Vec<FileInfo<'a>>> {
    paths
        .iter()
        .map(|path| -> AnyResult<FileInfo> {
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow!("file {:?} has an invalid file name", path))?;

            let file_name_lossy = file_name.to_string_lossy().to_string();

            let area_range = if let Some(matcher) = area_matcher {
                Some(try_find_area(&file_name_lossy, matcher)?)
            } else {
                None
            };

            if let Some((stem, ext)) = split_extension(path, secondary_ext_policy) {
                let ext_start_index = file_name_lossy.rfind(&ext.to_string_lossy().to_string());

                Ok(FileInfo {
                    path,
                    stem,
                    file_name: file_name_lossy,
                    area_range,
                    ext: Some(ext),
                    ext_start_index,
                })
            } else {
                // there is no file extension
                Ok(FileInfo {
                    path,
                    file_name: file_name_lossy,
                    stem: file_name,
                    area_range,
                    ext: None,
                    ext_start_index: None,
                })
            }
        })
        .collect::<AnyResult<_>>()
}

fn split_extension(
    path: &Path,
    secondary_ext_policy: SecondaryExtensionPolicy,
) -> Option<(&OsStr, OsString)> {
    let stem = path.file_stem()?;
    let ext = path.extension()?;

    let stem2 = Path::new(stem).file_stem()?;

    if stem2 == stem {
        // Single extension file (e.g. subtitle.srt)
        Some((stem, ext.to_os_string()))
    } else {
        // Double extension file (e.g. subtitle.en.srt)
        let ext2 = Path::new(stem).extension()?;
        let with_secondary = {
            let mut extensions = OsString::from(ext2);
            extensions.push(".");
            extensions.push(ext);
            Some((stem2, extensions))
        };

        match secondary_ext_policy {
            SecondaryExtensionPolicy::Always => with_secondary,
            SecondaryExtensionPolicy::Maybe => {
                // If the secondary extension includes a number (e.g. subtitle.2.srt) we ignore it
                // because it may contain the number part that matches the video file, and if it
                // is more than 3 characters long we also ignore it (because mpv will not auto detect
                // subtitle files when the secondary extension is longer than 3 characters).
                if NUMBER.is_match(&ext2.to_string_lossy())
                    || ext2.to_string_lossy().chars().count() > 3
                {
                    Some((stem, ext.to_os_string()))
                } else {
                    with_secondary
                }
            }
            SecondaryExtensionPolicy::Never => Some((stem, ext.to_os_string())),
        }
    }
}

fn try_find_area(file_name: &str, regex: &Regex) -> AnyResult<Range<usize>> {
    if let Some(m) = regex.find(file_name) {
        Ok(m.range())
    } else {
        bail!("failed to match regex {} on text: {}", regex, file_name);
    }
}
