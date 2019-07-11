use crate::config::Config;
use crate::util::*;
use regex::Regex;
use std::fmt::Debug;
use std::path::PathBuf;

lazy_static! {
    static ref NUMBER: Regex = Regex::new(r"\d+").unwrap();
}

lazy_static! {
    static ref EXTENSION: Regex = Regex::new(r"\.\w+?$").unwrap();
}

pub trait Action: Debug {
    fn apply(&self, matches: &[SubAndFile], config: &Config) -> Result<(), AnyError>;
}

#[derive(Builder, Debug)]
#[builder(pattern = "owned")]
pub struct SubTransformer {
    pub path: PathBuf,
    pub extensions: Vec<&'static str>,
    pub video_area: Option<Regex>,
    pub sub_area: Option<Regex>,
    pub actions: Vec<Box<Action>>,
}

#[derive(Debug)]
pub struct SubAndFile<'a> {
    pub sub_path: &'a str,
    pub sub_file_part: &'a str,
    pub sub_ext_part: &'a str,
    pub file_path: &'a str,
    pub file_file_part: &'a str,
    pub file_ext_part: Option<&'a str>,
}

impl<'a> SubAndFile<'a> {
    fn new(sub_path: &'a str, file_path: &'a str) -> SubAndFile<'a> {
        let (sub_file_part, sub_ext_part) =
            split_extension(sub_path).expect("sub file didn't have an extension");

        let (file_file_part, file_ext_part) = split_extension(file_path)
            .map(|(f, e)| (f, Some(e)))
            .unwrap_or((file_path, None));

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

impl SubTransformer {
    pub fn execute(&self, config: &Config) -> Result<(), AnyError> {
        let files_with_numbers = self.scan_number_files()?;
        let matched = self.match_files(&files_with_numbers)?;

        if matched.is_empty() {
            println!("found no match for any sub file");
        } else {
            for action in &self.actions {
                action.apply(&matched, config)?;
            }
        }

        Ok(())
    }

    fn scan_number_files(&self) -> Result<Vec<String>, AnyError> {
        let entries = std::fs::read_dir(&self.path)?.fold_results_vec()?;

        let mut files: Vec<String> = entries
            .iter()
            .map(|e| e.path().to_string_lossy().to_string())
            .filter(|p| NUMBER.is_match(&p))
            .collect();

        files.sort();

        Ok(files)
    }

    fn match_files<'a>(
        &self,
        files_with_numbers: &'a [String],
    ) -> Result<Vec<SubAndFile<'a>>, AnyError> {
        // Separate subtitle files from non-subtitle files.
        let (mut subs, mut others): (Vec<&String>, Vec<&String>) = files_with_numbers
            .iter()
            .partition(|file| self.extensions.iter().any(|ext| file.ends_with(ext)));

        // Find subs that already match their video files and return and remove them from subs
        // and others.
        let mut already_matched = extract_already_matched(&mut subs, &mut others);

        // Find the areas inside the paths that match the area regular expressions.
        let sub_areas = find_areas(subs, &self.sub_area)?;
        let mut other_areas = find_areas(others, &self.video_area)?;

        // Match the subtitle and other paths where they have the same number in their areas.
        let mut sub_and_files: Vec<SubAndFile<'a>> = sub_areas
            .iter()
            .filter_map(|sub| {
                let num = NUMBER.find(sub.area).map(|m| m.as_str())?;
                let num = num.parse::<u32>().unwrap().to_string();  // remove leading zeroes

                let (index, target) = other_areas
                    .iter()
                    .enumerate()
                    .find(|(_, other)| other.area.contains(&num))?;

                let sub_and_file = Some(SubAndFile::new(sub.text, target.text));

                other_areas.remove(index);
                sub_and_file
            })
            .collect();

        sub_and_files.append(&mut already_matched);
        Ok(sub_and_files)
    }
}

fn extract_already_matched<'a>(
    subs: &mut Vec<&'a String>,
    others: &mut Vec<&'a String>,
) -> Vec<SubAndFile<'a>> {
    let mut already_matched: Vec<SubAndFile<'a>> = vec![];

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

fn split_extension<'a>(path: &'a str) -> Option<(&'a str, &'a str)> {
    let ext = EXTENSION.find(path)?.as_str();
    let file = path.split(ext).next()?;
    Some((file, ext))
}

fn find_areas<'a>(
    texts: Vec<&'a String>,
    area_matcher: &Option<Regex>,
) -> Result<Vec<TextAndArea<'a>>, AnyError> {
    texts
        .iter()
        .map(|text| -> Result<TextAndArea, AnyError> {
            let area = try_extract_area(text, area_matcher)?;
            Ok(TextAndArea { text, area })
        })
        .fold_results_vec()
}

fn try_extract_area<'a>(text: &'a str, regex: &Option<Regex>) -> Result<&'a str, AnyError> {
    if let Some(r) = regex {
        if let Some(m) = r.find(text) {
            Ok(m.as_str())
        } else {
            let message = format!("failed to match regex {} on text: {}", r, text);
            Err(message.to_simple_error_boxed())
        }
    } else {
        Ok(text)
    }
}

#[derive(Debug)]
pub struct TextAndArea<'a> {
    text: &'a str,
    area: &'a str,
}
