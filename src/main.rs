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

extern crate crates_io_api;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate serde_derive;
extern crate metacpan_api;
extern crate serde;
extern crate serde_json;
extern crate rubygems_api;

mod crates;
mod gems;
mod helpers;
mod perldist;
#[cfg(test)]
mod tests;
mod tmplwriter;
mod types;

use helpers::*;
use tmplwriter::*;

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

    let pkg_type = figure_out_provider(tmpl_type, &pkg_name)
        .map_err(|e| err_handler(&e))
            .unwrap();

    // We don't want to generate templates for packages that are
    // built-in into perl/ruby
    if is_built_in(&pkg_name, &pkg_type) {
        return;
    }

    let pkg_info = get_pkginfo(&pkg_name, &pkg_type)
        .map_err(|e| err_handler(&e))
        .unwrap();

    if is_update_ver || is_update_all {
        update_template(&pkg_info, is_update_all)
            .map_err(|e| err_handler(&e))
            .unwrap();
    } else {
        template_handler(&pkg_info, &pkg_type, force_overwrite, false)
            .map_err(|e| err_handler(&e))
            .unwrap();
    }
}
