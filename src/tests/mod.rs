use crates::*;
use env_logger::Builder;
use gems::*;
use helpers::*;
use perldist::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use tmplwriter::*;
use types::*;

#[test]
fn test_query_crate() {
    let pkg_info = crate_info("rubygems_api").unwrap();
    assert_eq!(pkg_info.homepage, "https://github.com/Cogitri/rubygems_api");
}

#[test]
fn test_query_gem() {
    let pkg_info = gem_info("ffi").unwrap();
    assert_eq!(pkg_info.license[0], "BSD-3-Clause");
}

#[test]
fn test_query_perldist() {
    let pkg_info = perldist_info("Moose").unwrap();
    assert_eq!(pkg_info.pkg_name, "perl-Moose")
}

#[test]
fn test_tmplwriter() {
    Builder::new().filter(Some("tmplgen"), log::LevelFilter::Warn).default_format_timestamp(false).init();

    let pkg_info = PkgInfo {
        pkg_name: "tmplgen".to_string(),
        version: "0.3.1".to_string(),
        description: "Void Linux template generator for language-specific package managers".to_string(),
        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
        license: vec!["GPL-3.0-or-later".to_string()],
        dependencies: None,
    };

    let test_path = "/tmp/tmplgen-tests";

    env::set_var("XBPS_DISTDIR", test_path);
    env::set_var("HOME", test_path);

    write_template(&pkg_info, true, &PkgType::Crate).unwrap();

    let mut tmpl_file = File::open(format!("{}/srcpkgs/tmplgen/template", test_path)).unwrap();

    let mut tmpl_string = String::new();

    tmpl_file.read_to_string(&mut tmpl_string).unwrap();

    assert_eq!(tmpl_string, include_str!("template_test"));
}

#[test]
fn test_provider_selector() {
    assert_eq!(figure_out_provider(None,"tmplgen").unwrap(), PkgType::Crate);
}