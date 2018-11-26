use crates::*;
use env_logger::Builder;
use gems::*;
use helpers::*;
use perldist::*;
use rubygems_api::GemRunDeps;
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
    Builder::new()
        .filter(Some("tmplgen"), log::LevelFilter::Error)
        .default_format_timestamp(false)
        .init();

    let pkg_info = PkgInfo {
        pkg_name: "tmplgen".to_string(),
        version: "0.3.1".to_string(),
        description: "Void Linux template generator for language-specific package managers"
            .to_string(),
        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
        license: vec!["GPL-3.0-or-later".to_string()],
        dependencies: None,
        sha: None,
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
    assert_eq!(
        figure_out_provider(None, "tmplgen").unwrap(),
        PkgType::Crate
    );

    assert_eq!(
        figure_out_provider(None, "ruby-progressbar").unwrap(),
        PkgType::Gem
    );

    assert_eq!(
        figure_out_provider(None, "Moose").unwrap(),
        PkgType::PerlDist
    );

    assert_eq!(
        figure_out_provider(Some(PkgType::Crate), "").unwrap(),
        PkgType::Crate
    )
}

#[test]
#[should_panic]
fn test_figure_out_provider_panic() {
    figure_out_provider(None, "ffi").unwrap();
    figure_out_provider(None, "dioaüsdioüaw").unwrap();
}

#[test]
fn test_built_in() {
    assert_eq!(is_built_in("File::Basename", &PkgType::PerlDist), true)
}

#[test]
fn test_empty_gem_dep() {
    let pkg_info = gem_info("ffi").unwrap();

    assert_eq!(pkg_info.dependencies.unwrap().run, None);
}

#[test]
fn test_gen_dep_string_split() {
    let dep_gem_vec = [
        "ruby-rspec-core>=3.8.0".to_string(),
        "ruby-rspec-expectations>=3.8.0".to_string(),
        "ruby-rspec-mocks>=3.8.0".to_string(),
    ];

    let dep_gem_string = gen_dep_string(&dep_gem_vec, &PkgType::Gem).unwrap();

    assert_eq!(
        dep_gem_string.lines().last().unwrap(),
        " ruby-rspec-mocks>=3.8.0"
    );

    let dep_perldist_vec = [
        "File::Basename".to_string(),
        "parent".to_string(),
        "perl".to_string(),
        "JSON::PP".to_string(),
        "ExtUtils::MakeMaker".to_string(),
        "Dist::CheckConflicts".to_string(),
    ];

    let dep_perldist_string = gen_dep_string(&dep_perldist_vec, &PkgType::PerlDist).unwrap();

    assert_eq!(
        dep_perldist_string.lines().last().unwrap(),
        " perl-Dist-CheckConflicts"
    )
}

#[test]
fn test_crate_check_native_deps() {
    assert_eq!(
        &check_native_deps("openssl").unwrap().unwrap().make.unwrap()[0],
        "libressl-devel"
    )
}

//TODO: Improve the below test to test recursive deps
#[test]
fn test_gem_dep_graph() {
    assert!(gem_dep_graph("ffi").is_ok())
}

//TODO: Improve the below test to test recursive deps
#[test]
fn test_perl_dep_graph() {
    assert!(perldist_dep_graph("Moose").is_ok())
}

#[test]
fn test_determine_gem_run_deps() {
    let rubygem_deps = vec![
        GemRunDeps { name: "dep1".to_string(), requirements: ">= 0".to_string()},
        GemRunDeps { name: "dep2".to_string(), requirements: ">= 1".to_string()},
        GemRunDeps { name: "dep3".to_string(), requirements: "> 2".to_string()},
        GemRunDeps { name: "dep4".to_string(), requirements: "~> 1".to_string()}
    ];

    let mut dep_string = String::new();

    for x in rubygem_deps {
        dep_string.push_str(&determine_gem_run_deps(&x).unwrap());
        dep_string.push_str(" ");
    }

    assert_eq!(&dep_string, "ruby-dep1 ruby-dep2>=1 ruby-dep3>2 ruby-dep4>=1 ")
}