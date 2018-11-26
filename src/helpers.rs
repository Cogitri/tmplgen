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

        if (crate_status && gem_status)
            || (crate_status && perldist_status)
            || (gem_status && perldist_status)
        {
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
        crate_info(&pkg_name)
            .map_err(|e| {
                err_handler(&format!(
                    "Failed to info for the {:?} {}: {} ",
                    &pkg_type,
                    &pkg_name,
                    &e.to_string()
                ))
            })
            .unwrap()
    } else if pkg_type == &PkgType::PerlDist {
        if is_built_in(pkg_name, pkg_type) {
            return;
        }

        perldist_info(&pkg_name)
            .map_err(|e| {
                err_handler(&format!(
                    "Failed to info for the {:?} {}: {} ",
                    &pkg_type,
                    &pkg_name,
                    &e.to_string()
                ))
            })
            .unwrap()
    } else {
        if is_built_in(pkg_name, pkg_type) {
            return;
        }
        gem_info(pkg_name)
            .map_err(|e| {
                err_handler(&format!(
                    "Failed to info for the {:?} {}: {} ",
                    &pkg_type,
                    &pkg_name,
                    &e.to_string()
                ))
            })
            .unwrap()
    };

    write_template(&pkg_info, force_overwrite, &pkg_type).expect("Failed to write the template!");

    if pkg_type == &PkgType::Gem {
        gem_dep_graph(&pkg_name)
            .map_err(|e| {
                err_handler(&format!(
                    "Failed to query gem {}: {}",
                    pkg_name,
                    e.to_string()
                ))
            })
            .unwrap();
    } else if pkg_type == &PkgType::PerlDist {
        perldist_dep_graph(&pkg_name)
            .map_err(|e| {
                err_handler(&format!(
                    "Failed to query perldist {}: {}",
                    pkg_name,
                    e.to_string()
                ))
            })
            .unwrap();
    }
}

// Figure out where to write template files with `xdistdir`
pub fn xdist_files() -> Result<String, failure::Error> {
    let xdistdir = Command::new("sh").args(&["-c", "xdistdir"]).output()?;

    if !xdistdir.status.success() {
        return Err(format_err!(
            "xdistdir: exited with a non-0 exit code:\n {}",
            from_utf8(&xdistdir.stderr).unwrap()
        ));
    }

    Ok(format!(
        "{}/srcpkgs/",
        from_utf8(&xdistdir.stdout)?.replace("\n", "")
    ))
}

// Generic function to handle recursive deps.
pub fn recursive_deps(deps: &[String], xdistdir: &str, pkg_type: &PkgType) {
    for x in deps {
        let tmpl_path = if pkg_type == &PkgType::Gem {
            format!("{}ruby-{}/template", xdistdir, x)
        } else if pkg_type == &PkgType::PerlDist {
            // We don't write templates for modules, but only
            // for distributions. As such we have to convert
            // the module's name to the distribution's name,
            // if we're handling a module
            let perl_client = metacpan_api::SyncClient::new();

            let dist = perl_client.get_dist(&x);

            if dist.is_ok() {
                format!(
                    "{}perl-{}/template",
                    xdistdir,
                    dist.unwrap().replace("::", "-")
                )
            } else {
                format!("{}perl-{}/template", xdistdir, x.replace("::", "-"))
            }
        } else {
            format!("{}{}/template", xdistdir, x)
        };

        debug!("Checking for template in {}...", &tmpl_path);

        if !Path::new(&tmpl_path).exists() {
            info!(
                "Dependency {} doesn't exist yet, writing a template for it...",
                x
            );
            template_handler(x, &pkg_type, false);
        } else {
            debug!("Dependency {} is already satisfied!", x);
        }
    }
}

pub fn check_string_len(pkg_name: &str, string: &str, string_type: &str) -> String {
    if string.len() >= 80 {
        warn!(
            "{} of package {} is longer than 80 characters, please cut as you see fit!",
            pkg_name, string_type
        );
    }

    string.to_string()
}

pub fn is_built_in(pkg_name: &str, pkg_type: &PkgType) -> bool {
    let data: BuiltIns = serde_json::from_str(include_str!("built_in.in")).unwrap();

    let built_ins = BuiltIns {
        perl: data.perl,
        ruby: data.ruby,
    };

    if pkg_type == &PkgType::Gem {
        for x in built_ins.ruby {
            if pkg_name == x.name {
                warn!(
                    "Gem {} is part of ruby, won't write a template for it!",
                    pkg_name
                );
                return true;
            }
        }
    } else if pkg_type == &PkgType::PerlDist {
        let pkg_name = pkg_name.replace("::", "-");

        for x in built_ins.perl {
            if pkg_name == x.name {
                warn!(
                    "Perl distribution {} is part of perl, won't write a template for it!",
                    pkg_name
                );
                return true;
            }
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
    } else if matches.value_of("tmpltype").unwrap_or_default() == "perldist" {
        Some(PkgType::PerlDist)
    } else {
        None
    };

    let crate_name = String::from(matches.value_of("PKGNAME").unwrap());

    let force_overwrite = matches.is_present("force");

    let is_verbose = matches.is_present("verbose");

    let is_debug = matches.is_present("debug");

    (crate_name, tmpl_type, force_overwrite, is_verbose, is_debug)
}

pub fn gen_dep_string(dep_vec: &[String], pkg_type: &PkgType) -> Result<String, Error> {
    let mut dep_string = String::new();

    for x in dep_vec {
        let after_string = "".to_string() + &dep_string + x;

        let last_line_ln = after_string.lines().last().unwrap_or_default().len();

        // If the string with the new dep added _plus_ the {make,host,}depends=""
        // is longer than 80 chars, we want to split the line and insert a leading
        // space to the new line.
        // Otherwise, we want to add a space to the string (to seperate two deps),
        // but we don't want to introduce leading whitespace
        if &last_line_ln >= &65 {
            dep_string.push_str("\n ");
        } else if &last_line_ln > &x.len() {
            dep_string.push_str(" ");
        }

        if pkg_type == &PkgType::PerlDist {
            if x == "perl" {
            } else {
                dep_string.push_str(&("perl-".to_string() + &x.replace("::", "-")));
            }
        } else {
            dep_string.push_str(x);
        }
    }

    Ok(dep_string)
}

// TODO: Doing it this way means that all error using this function will show up as "ERROR tmplgen::helpers" in env_logger
pub fn err_handler(err_string: &str) {
    error!("{:?}", err_string);
    exit(1);
}
