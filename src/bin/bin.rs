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

use clap::{App, YamlLoader};
use env_logger::Builder;
use libtmplgen::*;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use log::{error, warn};

#[cfg(test)]
mod tests;

pub(crate) struct BinOptions {
    pub pkg_name: String,
    pub tmpl_type: Option<PkgType>,
    pub force_overwrite: bool,
    pub verbose: bool,
    pub debug: bool,
    pub update_ver: bool,
    pub update_all: bool,
    pub no_prefix: bool,
}

fn main() {
    let bin_options = help_string();

    set_up_logging(bin_options.debug, bin_options.verbose);

    // This isn't so very pretty, especially since main() can return Result since Rust 2018,
    // but we need this for pretty error messages via `env_logger`.
    actual_work(&bin_options)
        .map_err(|e| {
            error!("{}", e.to_string());
            std::process::exit(1);
        })
        .unwrap();
}

fn actual_work(opts: &BinOptions) -> Result<(), Error> {
    if opts.update_ver && opts.update_all {
        warn!("Specified both -u and -U! Will ignore -u");
    }

    let mut tmpl_builder = TmplBuilder::new(&opts.pkg_name);

    if opts.tmpl_type.is_some() {
        tmpl_builder.set_type(opts.tmpl_type.unwrap());
    } else {
        tmpl_builder.get_type()?;
    }

    if tmpl_builder.is_built_in()? {
        return Err(Error::BuiltIn(tmpl_builder.pkg_name.clone()));
    }

    if opts.no_prefix {
        let mut pkg_info = tmpl_builder.get_info()?.pkg_info.clone().unwrap();
        pkg_info.pkg_name = pkg_info
            .pkg_name
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

    let template = if opts.update_ver || opts.update_all {
        if Path::new(&xdist_template_path).exists() {
            let mut template_file = File::open(&xdist_template_path)?;
            let mut template_string = String::new();
            template_file.read_to_string(&mut template_string)?;

            tmpl_builder.update(
                &Template {
                    inner: template_string,
                    name: opts.pkg_name.clone(),
                },
                opts.update_all,
            )
        } else {
            return Err(Error::TmplUpdater(format!(
                "Can't update non-existing template {}",
                &tmpl_builder.pkg_info.unwrap().pkg_name
            )));
        }
    } else if Path::new(&xdist_template_path).exists() && !opts.force_overwrite {
        return Err(Error::TmplWriter(format!(
            "Won't overwrite existing template '{}' without `--force`!",
            &xdist_template_path,
        )));
    } else {
        tmpl_builder.generate(!opts.no_prefix)
    };

    create_dir_all(&xdist_template_path.replace("/template", ""))?;

    let mut file = File::create(&xdist_template_path)?;
    file.write_all(template?.inner.as_bytes())?;

    // We don't want to generate recursive deps for crates, as they don't have any!
    if tmpl_builder.pkg_type.unwrap() == PkgType::Crate {
        return Ok(());
    }

    if tmpl_builder
        .pkg_info
        .as_ref()
        .unwrap()
        .dependencies
        .is_some()
    {
        let dep_template_vec = tmpl_builder.gen_deps(Some(&format!("{}/srcpkgs", xdist_dir()?)));

        if dep_template_vec.is_ok() {
            for x in dep_template_vec.unwrap() {
                let xdist_template_path = format!("{}/srcpkgs/{}/template", xdist_dir()?, x.name,);

                create_dir_all(&xdist_template_path.replace("/template", ""))?;

                let mut file = File::create(&xdist_template_path)?;
                file.write_all(x.inner.as_bytes())?;
            }
        } else {
            return Err(Error::RecDeps {
                pkg_name: opts.pkg_name.clone(),
                err: dep_template_vec.err().unwrap().to_string(),
            });
        }
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
fn help_string() -> BinOptions {
    let help_yaml =
        YamlLoader::load_from_str(include_str!(concat!(env!("OUT_DIR"), "/cli_gen.yml"))).unwrap();
    let matches = App::from_yaml(&help_yaml[0]).get_matches();

    let tmpl_type = if matches.value_of("tmpltype").unwrap_or_default() == "crate" {
        Some(PkgType::Crate)
    } else if matches.value_of("tmpltype").unwrap_or_default() == "gem" {
        Some(PkgType::Gem)
    } else if matches.value_of("tmpltype").unwrap_or_default() == "perldist" {
        Some(PkgType::PerlDist)
    } else {
        None
    };

    let pkg_name = String::from(matches.value_of("PKGNAME").unwrap());

    let force_overwrite = matches.is_present("force");

    let verbose = matches.is_present("verbose");

    let debug = matches.is_present("debug");

    let update_ver = matches.is_present("update");

    let update_all = matches.is_present("update_all");

    let no_prefix = matches.is_present("no_prefix");

    BinOptions {
        pkg_name,
        tmpl_type,
        force_overwrite,
        verbose,
        debug,
        update_ver,
        update_all,
        no_prefix,
    }
}

fn xdist_dir() -> Result<String, Error> {
    let xdist_env = std::env::var_os("XBPS_DISTDIR");

    if xdist_env.is_none() {
        return Err(libtmplgen::Error::Xdist("Couldn't get XBPS_DISTDIR variable, please set it to where you want to write the template to!".to_string()));
    }

    let unclean_dir = std::str::from_utf8(xdist_env.unwrap().as_bytes())?.to_string();

    if unclean_dir.contains('~') {
        let home_dir = std::env::var("HOME");

        if home_dir.is_ok() {
            Ok(unclean_dir.replace("~", &home_dir.unwrap()))
        } else {
            Err(Error::Xdist(
                "Please either replace '~' with your homepath in XBPS_DISTDIR or export HOME"
                    .to_string(),
            ))
        }
    } else {
        Ok(unclean_dir)
    }
}
