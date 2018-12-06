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

use clap::{load_yaml, App};
use env_logger::Builder;
use libtmplgen::*;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use log::{error, warn};

#[cfg(test)]
mod tests;

fn main() {
    // This isn't so very pretty, especially since main() can return Result since Rust 2018,
    // but we need this for pretty error messages via `env_logger`.
    actual_work()
        .map_err(|e| {
            error!("{}", e.to_string());
            std::process::exit(1);
        })
        .unwrap();
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
    let no_prefix = help_tuple.7;

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

    if no_prefix {
        let mut pkg_info = tmpl_builder.get_info()?.pkg_info.clone().unwrap();
        pkg_info.pkg_name = pkg_info.pkg_name
            .replace("perl-", "")
            .replace("ruby-", "")
            .replace("rust-", "");
        tmpl_builder.set_info(pkg_info.to_owned());
    } else {
        tmpl_builder.get_info()?;
    }

    let xdist_template_path = format!(
        "{}/srcpkgs/{}/template",
        xdist_dir()?,
        tmpl_builder.pkg_info.as_ref().unwrap().pkg_name
    );

    let template = if is_update_ver || is_update_all {
        if Path::new(&xdist_template_path).exists() {
            let mut template_file = File::open(&xdist_template_path)?;
            let mut template_string = String::new();
            template_file.read_to_string(&mut template_string)?;

            tmpl_builder.update(
                &Template {
                    inner: template_string,
                    name: pkg_name.clone(),
                },
                is_update_all,
            )
        } else {
            return Err(Error::TmplUpdater(format!(
                "Can't update non-existing template {}",
                &tmpl_builder.pkg_info.unwrap().pkg_name
            )));
        }
    } else if Path::new(&xdist_template_path).exists() && !force_overwrite {
        return Err(Error::TmplWriter(format!(
            "Won't overwrite existing template '{}' without `--force`!",
            &xdist_template_path,
        )));
    } else {
        tmpl_builder.generate(!no_prefix)
    };

    create_dir_all(&xdist_template_path.replace("/template", ""))?;

    let mut file = File::create(&xdist_template_path)?;
    file.write_all(template?.inner.as_bytes())?;

    let deps = tmpl_builder.get_deps();

    if deps.is_ok() {
        let deps_vec = &deps.as_ref().unwrap().deps;
        if deps_vec.is_some() {
            let dep_template_vec = deps
                .as_ref()
                .unwrap()
                .gen_deps(Some(&format!("{}/srcpkgs", xdist_dir()?)));
            if dep_template_vec.is_ok() {
                for x in dep_template_vec.unwrap() {
                    let xdist_template_path =
                        format!("{}/srcpkgs/{}/template", xdist_dir()?, x.name,);

                    create_dir_all(&xdist_template_path.replace("/template", ""))?;

                    let mut file = File::create(&xdist_template_path)?;
                    file.write_all(x.inner.as_bytes())?;
                }
            } else {
                return Err(Error::RecDeps {
                    pkg_name,
                    err: dep_template_vec.err().unwrap().to_string(),
                });
            }
        } else {
            return Ok(());
        }
    } else {
        return Err(Error::RecDeps {
            pkg_name,
            err: deps.err().unwrap().to_string(),
        });
    }

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
fn help_string() -> (String, Option<PkgType>, bool, bool, bool, bool, bool, bool) {
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

    let no_prefix = matches.is_present("no_prefix");

    (
        crate_name,
        tmpl_type,
        force_overwrite,
        is_verbose,
        is_debug,
        update_ver_only,
        update_all,
        no_prefix,
    )
}

fn xdist_dir() -> Result<String, Error> {
    let xdist_env = std::env::var_os("XBPS_DISTDIR");

    if xdist_env.is_none() {
        return Err(libtmplgen::Error::XdistError("Couldn't get XBPS_DISTDIR variable, please set it to where you want to write the template to!".to_string()));
    }

    let unclean_dir = std::str::from_utf8(xdist_env.unwrap().as_bytes())?.to_string();

    if unclean_dir.contains('~') {
        let home_dir = std::env::var("HOME");

        if home_dir.is_ok() {
            Ok(unclean_dir.replace("~", &home_dir.unwrap()))
        } else {
            Err(Error::XdistError(
                "Please either replace '~' with your homepath in XBPS_XDISTDIR or export HOME"
                    .to_string(),
            ))
        }
    } else {
        Ok(unclean_dir)
    }
}
