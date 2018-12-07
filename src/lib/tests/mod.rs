use super::crates::*;
use super::gems::*;
use super::helpers::*;
use super::perldist::*;
use super::types::*;
use rubygems_api::GemRunDeps;
use std::env::set_var;

fn set_env() {
    set_var("XBPS_DISTDIR", "/tmp/tmplgen-tests");
    set_var("HOME", "/tmp/tmplgen-tests");
    set_var("GIT_AUTHOR_NAME", "tmplgentests");
    set_var("GIT_AUTHOR_EMAIL", "tmplgentests@github.com")
}

#[test]
fn test_query_crate() {
    let mut tmpl_builder = TmplBuilder::new("rubygems_api");
    tmpl_builder.set_type(PkgType::Crate).get_info().unwrap();
    assert_eq!(
        tmpl_builder.pkg_info.unwrap().homepage,
        "https://github.com/Cogitri/rubygems_api"
    );
}

#[test]
fn test_query_gem() {
    let mut tmpl_builder = TmplBuilder::new("ffi");
    tmpl_builder.set_type(PkgType::Gem).get_info().unwrap();
    assert_eq!(tmpl_builder.pkg_info.unwrap().license.unwrap()[0], "BSD-3-Clause");
}

#[test]
fn test_query_perldist() {
    let mut tmpl_builder = TmplBuilder::new("Moose");
    tmpl_builder.set_type(PkgType::PerlDist).get_info().unwrap();
    assert_eq!(tmpl_builder.pkg_info.unwrap().pkg_name, "perl-Moose")
}

#[test]
fn test_tmplwriter_correctness() {
    set_env();

    let pkg_info_crate = PkgInfo {
        pkg_name: "rust-tmplgen".to_string(),
        version: "0.3.1".to_string(),
        description: Some("Void Linux template generator for language-specific package managers"
            .to_string()),
        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
        license: Some(vec!["GPL-3.0-or-later".to_string()]),
        dependencies: None,
        sha: "dummy_sha".to_string(),
        download_url: Some(
            "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
        ),
    };

    let tmpl_string_crate = TmplBuilder::from_pkg_info(pkg_info_crate)
        .set_type(PkgType::Crate)
        .generate(true)
        .unwrap();

    assert_eq!(
        tmpl_string_crate.inner,
        include_str!("template_test_crate.in")
    );

    let pkg_info_perl = PkgInfo {
        pkg_name: "perl-Moose".to_string(),
        version: "2.2011".to_string(),
        description: Some("A postmodern object system for Perl 5".to_string()),
        homepage: "http://moose.perl.org/".to_string(),
        license: Some(vec!["perl_5".to_string()]),
        dependencies: Some(Dependencies {
            host: Some(vec!["perl".to_string()]),
            make: Some(vec![
                "JSON::PP".to_string(),
                "ExtUtils::MakeMaker".to_string(),
                "perl".to_string(),
                "Dist::CheckConflicts".to_string(),
            ]),
            run: Some(vec![
                "Devel::PartialDump".to_string(),
                "Data::OptList".to_string(),
                "Class::Load::XS".to_string(),
                "Params::Util".to_string(),
                "Sub::Identify".to_string(),
                "parent".to_string(),
                "Package::DeprecationManager".to_string(),
                "Scalar::Util".to_string(),
                "Carp".to_string(),
                "Eval::Closure".to_string(),
                "Data::OptList".to_string(),
                "Package::Stash::XS".to_string(),
                "Sub::Name".to_string(),
                "List::Util".to_string(),
                "Module::Runtime".to_string(),
                "Devel::OverloadInfo".to_string(),
                "perl".to_string(),
                "Sub::Exporter".to_string(),
                "warnings".to_string(),
                "Devel::StackTrace".to_string(),
                "Devel::GlobalDestruction".to_string(),
                "Package::Stash".to_string(),
                "Try::Tiny".to_string(),
                "MRO::Compat".to_string(),
                "Module::Runtime::Conflicts".to_string(),
                "Dist::CheckConflicts".to_string(),
                "strict".to_string(),
                "Class::Load".to_string(),
            ]),
        }),
        sha: "dummy_sha".to_string(),
        download_url: Some(
            "https://cpan.metacpan.org/authors/id/E/ET/ETHER/Moose-${version}.tar.gz".to_string(),
        ),
    };

    let tmpl_string_perl = TmplBuilder::from_pkg_info(pkg_info_perl)
        .set_type(PkgType::PerlDist)
        .generate(true)
        .unwrap();

    assert_eq!(
        tmpl_string_perl.inner,
        include_str!("template_test_perl.in")
    );
}

#[test]
fn test_provider_selector() {
    assert_eq!(
        TmplBuilder::new("tmplgen")
            .get_type()
            .unwrap()
            .pkg_type
            .unwrap(),
        PkgType::Crate
    );

    assert_eq!(
        TmplBuilder::new("ruby-progressbar")
            .get_type()
            .unwrap()
            .pkg_type
            .unwrap(),
        PkgType::Gem
    );

    assert_eq!(
        TmplBuilder::new("Moose")
            .get_type()
            .unwrap()
            .pkg_type
            .unwrap(),
        PkgType::PerlDist
    );
}

#[test]
#[should_panic]
fn test_figure_out_provider_panic() {
    TmplBuilder::new("ffi")
        .get_type()
        .unwrap()
        .pkg_type
        .unwrap();
    TmplBuilder::new("hdusapiduwipa")
        .get_type()
        .unwrap()
        .pkg_type
        .unwrap();
}

#[test]
fn test_built_in() {
    assert_eq!(
        TmplBuilder::new("File::Basename")
            .set_type(PkgType::PerlDist)
            .is_built_in()
            .unwrap(),
        true
    )
}

#[test]
fn test_empty_gem_dep() {
    let mut pkg_info = TmplBuilder::new("ffi");
    let pkg_info = pkg_info
        .set_type(PkgType::Gem)
        .get_info()
        .unwrap()
        .pkg_info
        .as_ref()
        .unwrap();

    assert_eq!(pkg_info.dependencies.as_ref().unwrap().run, None);
}

#[test]
fn test_gen_dep_string_split() {
    let dep_gem_vec = [
        "ruby-rspec-core>=3.8.0".to_string(),
        "ruby-rspec-expectations>=3.8.0".to_string(),
        "ruby-rspec-mocks>=3.8.0".to_string(),
    ];

    let dep_gem_string = gen_dep_string(&dep_gem_vec, PkgType::Gem);

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

    let dep_perldist_string = gen_dep_string(&dep_perldist_vec, PkgType::PerlDist);

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
    set_env();
    assert!(gem_dep_graph("ffi").is_ok())
}

//TODO: Improve the below test to test recursive deps
#[test]
fn test_perl_dep_graph() {
    set_env();
    assert!(perldist_dep_graph("Moose").is_ok())
}

#[test]
fn test_determine_gem_run_deps() {
    let rubygem_deps = vec![
        GemRunDeps {
            name: "dep1".to_string(),
            requirements: ">= 0".to_string(),
        },
        GemRunDeps {
            name: "dep2".to_string(),
            requirements: ">= 1".to_string(),
        },
        GemRunDeps {
            name: "dep3".to_string(),
            requirements: "> 2".to_string(),
        },
        GemRunDeps {
            name: "dep4".to_string(),
            requirements: "~> 1".to_string(),
        },
    ];

    let mut dep_string = String::new();

    for x in rubygem_deps {
        dep_string.push_str(&determine_gem_run_deps(&x));
        dep_string.push_str(" ");
    }

    assert_eq!(
        &dep_string,
        "ruby-dep1 ruby-dep2>=1 ruby-dep3>2 ruby-dep4>=1 "
    )
}

#[test]
fn test_correct_license() {
    assert_eq!(correct_license("GPL-1.0+"), "GPL-1.0-or-later".to_string());
    assert_eq!(
        correct_license("perl_5"),
        "Artistic-1.0-Perl, GPL-1.0-or-later".to_string()
    );
}

#[test]
fn test_template_updater() {
    set_env();

    let pkg_info_good = PkgInfo {
        pkg_name: "rust-tmplgen".to_string(),
        version: "0.3.1".to_string(),
        description: Some("Void Linux template generator for language-specific package managers"
            .to_string()),
        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
        license: Some(vec!["GPL-3.0-or-later".to_string()]),
        dependencies: None,
        sha: "dummy_sha".to_string(),
        download_url: Some(
            "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
        ),
    };

    let pkg_info_bad = PkgInfo {
        pkg_name: "rust-tmplgen".to_string(),
        version: "0.2.9".to_string(),
        description: Some("gibberish".to_string()),
        homepage: "htt/ri/tmplgen".to_string(),
        license: Some(vec!["GPL-3.0-or-later".to_string()]),
        dependencies: None,
        sha: "dummy".to_string(),
        download_url: Some("This Shouldn't be here".to_string()),
    };

    let bad_tmpl = TmplBuilder::from_pkg_info(pkg_info_bad)
        .set_type(PkgType::Crate)
        .generate(true)
        .unwrap();

    let good_template = TmplBuilder::from_pkg_info(pkg_info_good.clone())
        .set_type(PkgType::Crate)
        .update(&bad_tmpl, true)
        .unwrap();

    assert_eq!(good_template.inner, include_str!("template_test_crate.in"));

    let pkg_info_ok = PkgInfo {
        pkg_name: "rust-tmplgen".to_string(),
        version: "0.2.9".to_string(),
        description: Some("Void Linux template generator for language-specific package managers"
            .to_string()),
        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
        license: Some(vec!["GPL-3.0-or-later".to_string()]),
        dependencies: None,
        sha: "dummy".to_string(),
        download_url: Some(
            "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
        ),
    };

    let ok_tmpl = TmplBuilder::from_pkg_info(pkg_info_ok)
        .set_type(PkgType::Crate)
        .generate(true)
        .unwrap();

    let good_templ = TmplBuilder::from_pkg_info(pkg_info_good)
        .set_type(PkgType::Crate)
        .update(&ok_tmpl, false)
        .unwrap();

    assert_eq!(good_templ.inner, include_str!("template_test_crate.in"));
}

#[test]
fn test_get_git_author() {
    set_env();

    assert_eq!(
        &get_git_author().unwrap(),
        "tmplgentests <tmplgentests@github.com>"
    );
}

/*
#[test]
#[should_panic]
fn test_template_updater_panic() {
    set_env();

    let pkg_info_panic = PkgInfo {
        pkg_name: "tmplgendsadwadsaijodaioj".to_string(),
        version: "0.3.1".to_string(),
        description: "Void Linux template generator for language-specific package managers"
            .to_string(),
        homepage: "https://github.com/Cogitri/tmplgen".to_string(),
        license: vec!["GPL-3.0-or-later".to_string()],
        dependencies: None,
        sha: "dummy_sha".to_string(),
        download_url: Some(
            "https://static.crates.io/crates/tmplgen/tmplgen-${version}.crate".to_string(),
        ),
    };

    update_template(&pkg_info_panic, true, false).unwrap();
}*/
