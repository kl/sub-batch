use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;

mod util;

#[test]
fn can_rename_sub_file_that_contains_invalid_utf8() {
    let dir = tempdir().unwrap();
    util::copy("./tests/rename_invalid_utf8", &dir).unwrap();

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("-y")
        .arg("rename")
        .assert()
        .success();

    let files = util::files_in(&dir);
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"test_01_fake_video.mp4".to_string()));
    assert!(files.contains(&"test_01_fake_video.srt".to_string()));
}

#[test]
fn can_change_timings_of_sub_files() {
    fn timings(sub: &str) -> Vec<(String, String)> {
        sub.lines()
            .filter(|l| l.contains(" --> "))
            .map(|l| {
                let split = l.split(" --> ").collect::<Vec<_>>();
                (split[0].to_string(), split[1].to_string())
            })
            .collect()
    }

    let dir = tempdir().unwrap();
    util::copy("./tests/time_subs_only", &dir).unwrap();
    let files = util::files_in(&dir);

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("time")
        .arg("100")
        .assert()
        .success();

    let first = files.iter().find(|f| f.contains("sub.srt")).unwrap();
    let first_text = std::fs::read_to_string(&dir.path().join(first)).unwrap();

    let first_t = timings(&first_text);
    assert_eq!(first_t[0].0, "00:02:33,100");
    assert_eq!(first_t[2].1, "00:02:44,750");

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("time")
        .arg("-100")
        .assert()
        .success();

    let second = files.iter().find(|f| f.contains("another.srt")).unwrap();
    let second_text = std::fs::read_to_string(&dir.path().join(second)).unwrap();

    let second_t = timings(&second_text);
    assert_eq!(second_t[0].0, "00:12:33,488");
    assert_eq!(second_t[1].1, "00:12:40,161");
}
