use anyhow::Result as AnyResult;
use regex::Regex;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::fs::DirEntry;
use std::io;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref NUMBER: Regex = Regex::new(r"\d+").unwrap();
}

static EXTENSIONS: &[&str] = &["ssa", "ass", "sub", "srt", "idx"];

#[derive(Debug)]
pub struct SubAndFile {
    pub sub_path: PathBuf,
    pub sub_file_part: OsString,
    pub sub_ext_part: OsString,
    pub file_path: PathBuf,
    pub file_file_part: OsString,
    pub file_ext_part: Option<OsString>,
}

impl SubAndFile {
    fn new(sub_path: impl Into<PathBuf>, file_path: impl Into<PathBuf>) -> SubAndFile {
        let sub_path = sub_path.into();
        let file_path = file_path.into();

        let (sub_file_part, sub_ext_part) =
            split_extension(&sub_path).expect("sub file didn't have an extension");

        let (file_file_part, file_ext_part) = split_extension(&file_path)
            .map(|(f, e)| (f, Some(e)))
            .unwrap_or((file_path.file_stem().expect("invalid file name"), None));

        let sub_file_part = sub_file_part.to_owned();
        let sub_ext_part = sub_ext_part.to_owned();
        let file_file_part = file_file_part.to_owned();
        let file_ext_part = file_ext_part.map(OsStr::to_owned);

        SubAndFile {
            sub_path,
            sub_file_part,
            sub_ext_part,
            file_path,
            file_file_part,
            file_ext_part,
        }
    }
}

pub struct ScanOptions<'a> {
    pub path: &'a Path,
    pub sub_area: Option<&'a Regex>,
    pub video_area: Option<&'a Regex>,
    pub sub_filter: Option<&'a Regex>,
    pub video_filter: Option<&'a Regex>,
}

pub fn scan(options: ScanOptions) -> AnyResult<Vec<SubAndFile>> {
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
    std::fs::read_dir(&path)?.collect::<io::Result<Vec<DirEntry>>>()
}

fn match_files(
    options: &ScanOptions,
    files_with_numbers: &[PathBuf],
) -> AnyResult<Vec<SubAndFile>> {
    // Separate subtitle files from non-subtitle files.
    let (subs, others): (Vec<&PathBuf>, Vec<&PathBuf>) =
        files_with_numbers.iter().partition(|file| {
            let ext = file.extension().and_then(OsStr::to_str).unwrap_or_default();
            EXTENSIONS.contains(&ext)
        });

    // Remove files that don't match the filters.
    let mut subs = subs
        .into_iter()
        .filter(|sub| regex_matches_file_name(options.sub_filter, sub))
        .collect();
    let mut others = others
        .into_iter()
        .filter(|other| regex_matches_file_name(options.video_filter, other))
        .collect();

    // Find subs that already match their video files and return and remove them from subs
    // and others.
    let mut already_matched = extract_already_matched(&mut subs, &mut others);

    // Find the areas inside the paths that match the area regular expressions.
    let sub_areas = find_areas(subs, &options.sub_area)?;
    let mut other_areas = find_areas(others, &options.video_area)?;

    // Match the subtitle and other paths where they have the same number in their areas.
    let mut sub_and_files: Vec<SubAndFile> = sub_areas
        .iter()
        .filter_map(|sub| {
            let num = NUMBER.find(&sub.area).map(|m| m.as_str())?;
            let num = num.parse::<u32>().unwrap().to_string(); // remove leading zeroes

            let (index, target) = other_areas
                .iter()
                .enumerate()
                .find(|(_, other)| other.area.contains(&num))?;

            let sub_and_file = Some(SubAndFile::new(sub.path, target.path));

            other_areas.remove(index);
            sub_and_file
        })
        .collect();

    sub_and_files.append(&mut already_matched);
    Ok(sub_and_files)
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

fn extract_already_matched(
    subs: &mut Vec<&PathBuf>,
    others: &mut Vec<&PathBuf>,
) -> Vec<SubAndFile> {
    let mut already_matched: Vec<SubAndFile> = vec![];

    subs.retain(|sub| {
        let (sub_file_part, _) = split_extension(sub).expect("sub file didn't have an extension");

        if let Some((index, _)) = others.iter().enumerate().find(|(_, other)| {
            // We may not have an extension, if not compare entire file path.
            split_extension(other)
                .map(|(other_file_part, _)| sub_file_part == other_file_part)
                .unwrap_or_else(|| sub_file_part == **other)
        }) {
            already_matched.push(SubAndFile::new(sub, others.remove(index)));
            false
        } else {
            true
        }
    });

    already_matched
}

fn split_extension(path: &Path) -> Option<(&OsStr, &OsStr)> {
    Some((path.file_stem()?, path.extension()?))
}

fn find_areas<'a>(
    paths: Vec<&'a PathBuf>,
    area_matcher: &Option<&Regex>,
) -> AnyResult<Vec<PathAndArea<'a>>> {
    paths
        .iter()
        .map(|path| -> AnyResult<PathAndArea> {
            let area = try_extract_area(path, area_matcher)?;
            Ok(PathAndArea { path, area })
        })
        .collect::<AnyResult<_>>()
}

fn try_extract_area(path: &Path, regex: &Option<&Regex>) -> AnyResult<String> {
    let name = try_extract_file_name(path)?;

    if let Some(r) = regex {
        if let Some(m) = r.find(&name) {
            Ok(m.as_str().into())
        } else {
            bail!("failed to match regex {} on text: {}", r, name);
        }
    } else {
        Ok(name)
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
}
