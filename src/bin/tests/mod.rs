//This file is part of tmplgen
//
//tmplgen is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//tmplgen is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with tmplgen.  If not, see <http://www.gnu.org/licenses/>.

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_bin_gen() {
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
fn test_bin_update() {
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

    Command::main_binary()
        .unwrap()
        .args(&["-U", "tmplgen"])
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
