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
//along with Foobar.  If not, see <http://www.gnu.org/licenses/>.

extern crate crates_io_api;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate env_logger;

mod crates;
mod gems;
mod helpers;
#[cfg(test)]
mod tests;
mod tmplwriter;
mod types;

use clap::App;
use helpers::*;
use types::PkgType;

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

fn main() {
    let help_tuple = help_string();
    let pkg_name = help_tuple.0;
    let tmpl_type = help_tuple.1;
    let force_overwrite = help_tuple.2;
    let is_verbose = help_tuple.3;
    let is_debug = help_tuple.4;

    if is_debug {
        let env =
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "tmplgen=debug");
        env_logger::Builder::from_env(env).init();
    } else if is_verbose {
        let env =
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "tmplgen=info");
        env_logger::Builder::from_env(env).init();
    } else {
        env_logger::init();
    }

    let pkg_type = figure_out_provider(tmpl_type, &pkg_name).unwrap();

    template_handler(pkg_name, &pkg_type, force_overwrite);
}
