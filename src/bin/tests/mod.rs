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

use crate::*;
use assert_cmd::prelude::*;
use std::env::set_var;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_bin_gen() {
    let dir = tempdir().unwrap();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
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

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(&["tmplgen"])
        .env_clear()
        .env("XBPS_DISTDIR", dir.path().join("tmplgen-tests/"))
        .env("GIT_AUTHOR_NAME", "tmplgentests")
        .env("GIT_AUTHOR_EMAIL", "tmplgen@tests.de")
        .assert()
        .success();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
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

#[test]
fn test_xdist_dir() {
    set_var("XBPS_DISTDIR", "~/void-packages");
    set_var("HOME", "/home/tmplgen");

    assert_eq!(
        xdist_dir().unwrap(),
        "/home/tmplgen/void-packages".to_string()
    )
}

#[test]
fn test_main_worker() {
    let dir = tempdir().unwrap();

    set_var("XBPS_DISTDIR", dir.path());
    set_var("GIT_AUTHOR_NAME", "tmplgentests");
    set_var("GIT_AUTHOR_EMAIL", "tmplgen@tests.de");

    let mut opts = BinOptions {
        pkg_name: "tmplgen".to_string(),
        tmpl_type: Some(PkgType::Crate),
        force_overwrite: false,
        verbose: false,
        debug: false,
        update_all: false,
        update_ver: false,
        no_prefix: false,
    };

    actual_work(&opts).unwrap();

    opts.force_overwrite = true;

    opts.no_prefix = true;
    actual_work(&opts).unwrap();

    //opts.update_ver = true;
    //actual_work(&opts).unwrap();
    //opts.update_ver = false;

    //actual_work(&opts).unwrap();
    //opts.update_all = true;
    //actual_work(&opts).unwrap();

    dir.close().unwrap()
}
