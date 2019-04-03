use crate::config::Config;
use crate::util::AnyError;
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
        let matched = self.match_files(&files_with_numbers);

        for action in &self.actions {
            action.apply(&matched, config)?;
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

    fn match_files<'a>(&self, files_with_numbers: &'a [String]) -> Vec<SubAndFile<'a>> {
        let (subs, others): (Vec<&String>, Vec<&String>) = files_with_numbers
            .iter()
            .partition(|file| self.extensions.iter().any(|ext| file.ends_with(ext)));

        subs.iter()
            .filter_map(|sub| {
                let area = try_extract_area(sub, &self.sub_area);
                let num = NUMBER.find(area).map(|m| m.as_str())?;

                others
                    .iter()
                    .find(|other| {
                        let area = try_extract_area(other, &self.video_area);
                        area.contains(num)
                    })
                    .map(|target| SubAndFile {
                        sub_path: sub,
                        file_path: target,
                    })
            })
            .collect()
    }
}

/// Returns the area matched by the regex, or the entire &str if regex is None or the regex
/// did not match.
fn try_extract_area<'a>(text: &'a str, regex: &Option<Regex>) -> &'a str {
    if let Some(r) = regex {
        if let Some(m) = r.find(text) {
            return m.as_str();
        } else {
            eprintln!(
                "warning: failed to match regex {} on text: {}, using whole text instead",
                r, text
            )
        }
    }
    text
}
