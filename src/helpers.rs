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

use clap::App;
use crates::*;
use env_logger::Builder;
use gems::*;
use perldist::*;
use std::path::Path;
use std::process::{exit, Command};
use std::str::from_utf8;
use tmplwriter::*;
use types::*;

pub fn missing_field_s(field_name: &str) -> String {
    warn!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    String::from("")
}

pub fn missing_field_v(field_name: &str) -> Vec<String> {
    warn!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    vec![String::from("")]
}

// Figure out whether we're dealing with a crate or a gem if the user hasn't specified that.
// Errors out of a package with the name the user gave us can be found on both crates.io and
// rubygems.org
pub fn figure_out_provider(
    tmpl_type: Option<PkgType>,
    pkg_name: &str,
) -> Result<PkgType, failure::Error> {
    if tmpl_type.is_none() {
        let crate_status = crates_io_api::SyncClient::new()
            .get_crate(&pkg_name)
            .is_ok();

        let gem_status = rubygems_api::SyncClient::new().gem_info(&pkg_name).is_ok();

        let perldist_status = metacpan_api::SyncClient::new().perl_info(&pkg_name).is_ok();

        if (crate_status && gem_status) || (crate_status && perldist_status) || (gem_status && perldist_status) {
            Err(format_err!("Found a package matching the specified name on multiple platforms! Please explicitly choose one via the `-t` parameter!"))
        } else if crate_status {
            debug!("Determined the target package to be a crate");
            Ok(PkgType::Crate)
        } else if gem_status {
            debug!("Determined the target package to be a ruby gem");
            Ok(PkgType::Gem)
        } else if perldist_status {
            debug!("Determined the target package to be a perldist");
            Ok(PkgType::PerlDist)
        } else {
            Err(format_err!("Unable to determine what type of the target package! Make sure you've spelled the package name correctly!"))
        }
    } else {
        Ok(tmpl_type.unwrap())
    }
}

// Handle getting the necessary info and writing a template for it. Invoked every time a template
// should be written, useful for recursive deps.
pub fn template_handler(pkg_name: &str, pkg_type: &PkgType, force_overwrite: bool) {
    info!(
        "Generating template for package {} of type {:?}",
        &pkg_name, pkg_type
    );

    let pkg_info = if pkg_type == &PkgType::Crate {
        crate_info(&pkg_name).map_err(|e| err_handler(&format!("Failed to info for the {:?} {}: {} ", &pkg_type, &pkg_name, &e.to_string()))).unwrap()
    } else if pkg_type == &PkgType::PerlDist {
        perldist_info(&pkg_name).map_err(|e| err_handler(&format!("Failed to info for the {:?} {}: {} ", &pkg_type, &pkg_name, &e.to_string()))).unwrap()
    } else {
        if is_dist_gem(pkg_name) {
            return;
        }
        gem_info(pkg_name).map_err(|e| err_handler(&format!("Failed to info for the {:?} {}: {} ", &pkg_type, &pkg_name, &e.to_string()))).unwrap()
    };

    write_template(&pkg_info, force_overwrite, &pkg_type).expect("Failed to write the template!");

    if pkg_type == &PkgType::Gem {
        gem_dep_graph(&pkg_name, force_overwrite);
    }
}

// Figure out where to write template files with `xdistdir`
pub fn xdist_files() -> String {
    let xdistdir = Command::new("sh")
        .args(&["-c", "xdistdir"])
        .output()
        .expect("Couldn't execute xdistdir. Make sure you have xtools installed.");

    if !xdistdir.status.success() {
        error!(
            "xdistdir: exited with a non-0 exit code:\n {:?}",
            from_utf8(&xdistdir.stderr).unwrap()
        );

        exit(1);
    }

    format!(
        "{}/srcpkgs/",
        from_utf8(&xdistdir.stdout)
            .map_err(|e| err_handler(&e.to_string()))
            .unwrap()
            .replace("\n", "")
            .replace(
                "~",
                &std::env::var("HOME")
                    .expect("Please either replace '~' with your homepath or export HOME")
            ),
    )
}

// Generic function to handle recursive deps. Only used for gems as of now.
pub fn recursive_deps(
    deps: &[String],
    xdistdir: &str,
    pkg_type: &PkgType,
    force_overwrite: bool,
) {
    if force_overwrite {
        for x in deps {
            info!("Specified `-f`, will overwrite existing templates if they exists...");
            template_handler(x, &pkg_type, force_overwrite);
        }
    } else {
        for x in deps {
            let tmpl_path = if pkg_type == &PkgType::Gem {
                format!("{}ruby-{}/template", xdistdir, x)
            } else {
                format!("{}{}/template", xdistdir, x)
            };
            if !Path::new(&tmpl_path).exists() {
                info!(
                    "Dependency {} doesn't exist yet, writing a template for it...",
                    x
                );
                template_handler(x, &pkg_type, force_overwrite);
            } else {
                debug!("Dependency {} is already satisfied!", x);
            }
        }
    }
}

pub fn check_string_len(string: &str, string_type: &str) -> String {
    if string.len() >= 80 {
        warn!(
            "{} is longer than 80 characters, please cut as you see fit!",
            string_type
        );
    }

    string.to_string()
}

pub fn is_dist_gem(pkg_name: &str) -> bool {
    for x in include_str!("dist_gems.in").split_whitespace() {
        if pkg_name == x {
            error!(
                "Gem {} is part of ruby, won't write a template for it!",
                pkg_name
            );
            return true;
        }
    }

    false
}

pub fn set_up_logging(is_debug: bool, is_verbose: bool) {
    let mut builder = Builder::new();

    if is_debug {
        builder.filter(Some("tmplgen"), log::LevelFilter::Debug);
    } else if is_verbose {
        builder.filter(Some("tmplgen"), log::LevelFilter::Info);
    } else {
        builder.filter(Some("tmplgen"), log::LevelFilter::Warn);
    }

    builder.default_format_timestamp(false).init();

    if is_debug && is_verbose {
        warn!("Specified both --verbose and --debug! Will ignore --verbose.");
    }
}

// Print the help script if invoked without arguments or with `--help`/`-h`
pub fn help_string() -> (String, Option<PkgType>, bool, bool, bool) {
    let help_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(help_yaml).get_matches();

    let tmpl_type = if matches.value_of("tmpltype").unwrap_or_default() == "crate" {
        Some(PkgType::Crate)
    } else if matches.value_of("tmpltype").unwrap_or_default() == "gem" {
        Some(PkgType::Gem)
    } else {
        None
    };

    let crate_name = String::from(matches.value_of("PKGNAME").unwrap());

    let force_overwrite = matches.is_present("force");

    let is_verbose = matches.is_present("verbose");

    let is_debug = matches.is_present("debug");

    (crate_name, tmpl_type, force_overwrite, is_verbose, is_debug)
}

pub fn gen_dep_string(dep_vec: &[String]) -> String {
    let mut dep_string = String::new();

    for x in dep_vec {
        let after_string = "".to_string() + &dep_string + x;

        // If the string with the new dep added is longer than or equal to 80
        // chars we want
        if after_string.lines().last().unwrap().len() >= 80 {
            dep_string.push_str("\n")
        }

        dep_string.push_str(x);
        dep_string.push_str(" ");
    }

    dep_string
}

pub fn err_handler(err_string: &str) {
    error!("{:?}", err_string);
    exit(1);
}
