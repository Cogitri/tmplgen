use std::process::Command;
use assert_cmd::prelude::*;
use std::env;

#[test]
fn test_bin_run() {
    Command::main_binary()
        .unwrap()
        .args(&["tmplgen"])
        .env_clear()
        .env("XBPS_DISTDIR", format!("{:?}/tmplgen-tests", env::temp_dir()))
        .env("GIT_AUTHOR_NAME", "tmplgentests")
        .env("GIT_AUTHOR_EMAIL", "tmplgen@tests.de")
        .assert()
        .success();
}

#[test]
#[should_panic]
fn test_bad_env() {
    Command::cargo_bin("tmplgen").unwrap()
        .args(&["tmplgen"])
        .env_clear()
        .assert()
        .success();
}