use crates::*;
use gems::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use tmplwriter::*;
use types::*;

#[test]
fn test_query_crate() {
    let pkg_info = crate_info(&"rubygems_api".to_string()).unwrap();
    assert_eq!(pkg_info.homepage, "https://github.com/Cogitri/rubygems_api");
}

#[test]
fn test_query_gem() {
    let pkg_info = gem_info(&"ffi".to_string()).unwrap();
    assert_eq!(pkg_info.license[0], "BSD-3-Clause");
}

#[test]
fn test_tmplwriter() {
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

    write_template(&pkg_info, true, &PkgType::Crate).unwrap();

    let mut tmpl_file = File::open(format!("{}/srcpkgs/tmplgen/template", test_path)).unwrap();

    let mut tmpl_string = String::new();

    tmpl_file.read_to_string(&mut tmpl_string).unwrap();

    assert_eq!(tmpl_string, include_str!("template_test"));
}
