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

use std::path::Path;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use libtmplgen::*;
use clap::{App, load_yaml};
use env_logger::Builder;
use std::os::unix::ffi::OsStrExt;

use log::{error, warn};

fn main() {
    // This isn't so very pretty, especially since main() can return Result since Rust 2018,
    // but we need this for pretty error messages via `env_logger`.
    actual_work().map_err(|e| error!("{}", e.to_string())).unwrap_or_default();
}

fn actual_work() -> Result<(), Error> {
    let help_tuple = help_string();
    let pkg_name = help_tuple.0;
    let tmpl_type = help_tuple.1;
    let force_overwrite = help_tuple.2;
    let is_verbose = help_tuple.3;
    let is_debug = help_tuple.4;
    let is_update_ver = help_tuple.5;
    let is_update_all = help_tuple.6;

    set_up_logging(is_debug, is_verbose);

    if is_update_ver && is_update_all {
        warn!("Specified both -u and -U! Will ignore -u");
    }

    let mut tmpl_builder = TmplBuilder::new(&pkg_name);

    if tmpl_type.is_some() {
        tmpl_builder.set_type(tmpl_type.unwrap());
    } else {
        tmpl_builder.get_type()?;
    }

    if tmpl_builder.is_built_in()? {
        return Err(Error::BuiltIn(tmpl_builder.pkg_name.clone()));
    }

    let xdist_env = std::env::var_os("XBPS_DISTDIR");

    let xdist_dir = if xdist_env.is_some() {
        Ok(std::str::from_utf8(xdist_env.unwrap().as_bytes())?.to_string())
    } else {
        Err(libtmplgen::Error::XdistError("Couldn't get XBPS_DISTDIR variable, please set it to where you want to write the template to!".to_string()))
    };

    let xdist_template_path = format!("{}{}/template", xdist_dir?, tmpl_builder.pkg_info.as_ref().unwrap().pkg_name);

    let template = if is_update_ver || is_update_all {
        if Path::new(&xdist_template_path).exists() {
            let mut template_file = File::open(&xdist_template_path)?;
            let mut template_string = String::new();
            template_file.read_to_string(&mut template_string)?;

            tmpl_builder.get_info()?.update(&Template { inner: template_string }, is_update_all)
        } else {
            return Err(Error::TmplUpdater(format!("Can't update non-existing template {}", &tmpl_builder.pkg_info.unwrap().pkg_name)));
        }
    } else if Path::new(&xdist_template_path).exists() && !force_overwrite {
        return Err(Error::TmplWriter(format!(
            "Won't overwrite existing template '{}/template' without `--force`!",
            &xdist_template_path,
        )));
    } else {
        tmpl_builder.get_info()?.generate()
    };

    create_dir_all(&xdist_template_path.replace("/template", ""))?;

    let mut file = File::create(&xdist_template_path)?;
    file.write_all(template?.inner.as_bytes())?;


    /*
    if pkg_type == PkgType::Crate {
        return Ok(());
    }

    let dep_graph = if pkg_type == PkgType::Gem {
        gem_dep_graph(&pkg_name.replace("ruby-", ""))
    } else {
        perldist_dep_graph(&pkg_name.replace("perl-", ""))
    };

    if dep_graph.is_err() {
        warn!(
            "Failed to write templates for all recursive deps of {}! Error: {}",
            pkg_name,
            dep_graph.unwrap_err()
        );
    }*/

    Ok(())
}

fn set_up_logging(is_debug: bool, is_verbose: bool) {
    let mut builder = Builder::new();

    if is_debug {
        builder
            .filter_module("libtmplgen", log::LevelFilter::Debug)
            .filter_module("tmplgen", log::LevelFilter::Debug)
            // Also include what GET requests the below modules do, which
            // get the data we need for PkgInfo
            .filter_module("crates_io_api", log::LevelFilter::Trace)
            .filter_module("rubygems_api", log::LevelFilter::Debug)
            .filter_module("metacpan_api", log::LevelFilter::Debug);
    } else if is_verbose {
        builder
            .filter_module("libtmplgen", log::LevelFilter::Info)
            .filter_module("tmplgen", log::LevelFilter::Info);
    } else {
        builder
            .filter_module("libtmplgen", log::LevelFilter::Warn)
            .filter_module("tmplgen", log::LevelFilter::Warn);
    }

    builder.default_format_timestamp(false).init();

    if is_debug && is_verbose {
        warn!("Specified both --verbose and --debug! Will ignore --verbose.");
    }
}

// Print the help script if invoked without arguments or with `--help`/`-h`
fn help_string() -> (String, Option<PkgType>, bool, bool, bool, bool, bool) {
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

    let update_ver_only = matches.is_present("update");

    let update_all = matches.is_present("update_all");

    (
        crate_name,
        tmpl_type,
        force_overwrite,
        is_verbose,
        is_debug,
        update_ver_only,
        update_all,
    )
}

/*
/// Generic function to handle recursive deps.
///
/// # Errors
/// * Errors out if `template_handler` fails to run
pub(super) fn recursive_deps(deps: &[String], xdistdir: &str, pkg_type: PkgType) -> Result<(), Error> {
    for x in deps {
        // We want to ignore built-in deps
        if !is_built_in(x, pkg_type) {
            let tmpl_path = if pkg_type == PkgType::Gem {
                format!("{}ruby-{}/template", xdistdir, x)
            } else if pkg_type == PkgType::PerlDist {
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
                template_handler(&get_pkginfo(&x, pkg_type)?, pkg_type, false, true)?;
            } else {
                debug!("Dependency {} is already satisfied!", x);
            }
        }
    }
    Ok(())
}
*/