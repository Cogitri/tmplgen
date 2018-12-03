use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_bin_run() {
    let dir = tempdir().unwrap();

    Command::main_binary()
        .unwrap()
        .args(&["tmplgen"])
        .env_clear()
        .env("XBPS_DISTDIR", dir.path().join("tmplgen-tests/"))
        .env("GIT_AUTHOR_NAME", "tmplgentests")
        .env("GIT_AUTHOR_EMAIL", "tmplgen@tests.de")
        .assert()
        .success();

    dir.close().unwrap();
}

#[test]
#[should_panic]
fn test_bad_env() {
    Command::cargo_bin("tmplgen")
        .unwrap()
        .args(&["tmplgen"])
        .env_clear()
        .assert()
        .success();
}
