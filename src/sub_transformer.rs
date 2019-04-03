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
    pub file_path: &'a str,
}

impl SubAndFile<'_> {
    pub fn sub_ext(&self) -> &str {
        EXTENSION.find(self.sub_path).map(|m| m.as_str()).unwrap()
    }

    pub fn file_ext(&self) -> Option<&str> {
        EXTENSION.find(self.file_path).map(|m| m.as_str())
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
        let mut files: Vec<String> = std::fs::read_dir(&self.path)?
            .map(|f| f.unwrap().path().to_str().unwrap().to_string())
            .filter(|f| NUMBER.is_match(&f))
            .collect();

        files.sort();

        Ok(files)
    }

    fn match_files<'a>(
        &self,
        files_with_numbers: &'a [String],
    ) -> Result<Vec<SubAndFile<'a>>, AnyError> {
        // Separate subtitle files from non-subtitle files.
        let (subs, others): (Vec<&String>, Vec<&String>) = files_with_numbers
            .iter()
            .partition(|file| self.extensions.iter().any(|ext| file.ends_with(ext)));

        // Find the areas inside the paths that match the area regular expressions.
        let sub_areas = find_areas(subs, &self.sub_area)?;
        let mut other_areas = find_areas(others, &self.video_area)?;

        // Match the subtitle and other paths where they have the same number in their areas.
        Ok(sub_areas
            .iter()
            .filter_map(|sub| {
                let num = NUMBER.find(sub.area).map(|m| m.as_str())?;
                // Parse the number to remove any leading zeroes.
                let parsed = num.parse::<u32>().unwrap().to_string();

                let (index, target) = other_areas
                    .iter()
                    .enumerate()
                    .find(|(_, other)| other.area.contains(&parsed))?;

                let sub_and_file = Some(SubAndFile {
                    sub_path: sub.text,
                    file_path: target.text,
                });

                other_areas.remove(index);
                sub_and_file
            })
            .collect())
    }
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
