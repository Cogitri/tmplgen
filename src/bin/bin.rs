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

use libtmplgen::*;
use clap::{App, load_yaml};
use env_logger::Builder;

use log::{error, warn};

fn main() {
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

    let pkg_type = if tmpl_type.is_some() {
        tmpl_type.unwrap()
    } else {
        figure_out_provider(&pkg_name)
            .map_err(|e| err_handler(&e))
            .unwrap()
    };

    let pkg_info = get_pkginfo(&pkg_name, pkg_type)
        .map_err(|e| err_handler(&e))
        .unwrap();

    if is_update_ver || is_update_all {
        update_template(&pkg_info, is_update_all, force_overwrite)
            .map_err(|e| err_handler(&e))
            .unwrap();
    } else {
        template_handler(&pkg_info, pkg_type, force_overwrite, false)
            .map_err(|e| err_handler(&e))
            .unwrap();
    }
}

fn set_up_logging(is_debug: bool, is_verbose: bool) {
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

fn err_handler(error: &Error) {
    error!("{}", error.to_string());
    std::process::exit(1);
}