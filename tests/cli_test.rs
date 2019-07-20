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
