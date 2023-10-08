use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

mod util;

#[test]
fn can_rename_sub_file() {
    let dir = tempdir().unwrap();
    util::copy("./tests/rename", &dir).unwrap();

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("-y")
        .arg("rename")
        .assert()
        .success();

    let files = util::files_in(&dir);
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"sample-video-01.mp4".to_string()));
    assert!(files.contains(&"sample-video-01.srt".to_string()));
}

#[test]
fn rename_does_not_happen_when_filter_is_not_matching() {
    let dir = tempdir().unwrap();
    util::copy("./tests/rename", &dir).unwrap();

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("--filter-video")
        .arg("non-matching-regex")
        .arg("-y")
        .arg("rename")
        .assert()
        .failure();

    let files = util::files_in(&dir);
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"sample-video-01.mp4".to_string()));
    assert!(files.contains(&"sub01.srt".to_string()));
}

#[test]
fn can_rename_sub_file_that_contains_invalid_utf8() {
    let dir = tempdir().unwrap();
    util::copy("./tests/rename_invalid_utf8", &dir).unwrap();

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("-y")
        .arg("rename")
        .arg("--subarea")
        .arg("\\d{2}\\.srt")
        .assert()
        .success();

    let files = util::files_in(&dir);
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"test_01_fake_video.mp4".to_string()));
    assert!(files.contains(&"test_01_fake_video.srt".to_string()));
}

#[test]
fn can_change_timings_of_sub_files() {
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

#[test]
fn timings_do_not_change_when_filter_is_not_matching() {
    let dir = tempdir().unwrap();
    util::copy("./tests/time_subs_only", &dir).unwrap();
    let files = util::files_in(&dir);

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("--filter-sub")
        .arg("non-matching-regex")
        .arg("time")
        .arg("100")
        .assert()
        .failure();

    let first = files.iter().find(|f| f.contains("sub.srt")).unwrap();
    let first_text = std::fs::read_to_string(&dir.path().join(first)).unwrap();

    let first_t = timings(&first_text);
    assert_eq!(first_t[0].0, "00:02:33,000");
    assert_eq!(first_t[2].1, "00:02:44,650");
}

#[test]
fn can_run_alass_on_sub_file() {
    let dir = tempdir().unwrap();
    util::copy("./tests/dummy", &dir).unwrap();

    Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("-y")
        .arg("alass")
        .arg("--split-penalty 10")
        .assert()
        .success()
        .stdout(predicate::str::contains("shifted block of 1"));
}

#[test]
fn can_show_confirm_without_panicking() {
    // run commands without the -y switch so the confirm is shown and make sure that
    // we don't panic by checking that the program is still running after waiting a bit

    let dir = tempdir().unwrap();
    util::copy("./tests/dummy", &dir).unwrap();

    let mut spawn = Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("alass")
        .spawn()
        .expect("should not panic");

    thread::sleep(Duration::from_millis(100));
    assert!(
        spawn.try_wait().expect("wait error").is_none(),
        "command panicked"
    );
    let _ = spawn.kill();

    let dir = tempdir().unwrap();
    util::copy("./tests/rename_invalid_utf8", &dir).unwrap();

    let mut spawn = Command::cargo_bin("sub-batch")
        .unwrap()
        .current_dir(&dir)
        .arg("rename")
        .arg("--subarea")
        .arg("\\d{2}\\.srt")
        .spawn()
        .expect("should not panic");

    thread::sleep(Duration::from_millis(100));
    assert!(
        spawn.try_wait().expect("wait error").is_none(),
        "command panicked"
    );
    let _ = spawn.kill();
}

fn timings(sub: &str) -> Vec<(String, String)> {
    sub.lines()
        .filter(|l| l.contains(" --> "))
        .map(|l| {
            let split = l.split(" --> ").collect::<Vec<_>>();
            (split[0].to_string(), split[1].to_string())
        })
        .collect()
}
