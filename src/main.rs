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

mod crates;
mod gems;
mod helpers;
#[cfg(test)]
mod tests;
mod tmplwriter;
mod types;

use helpers::*;

fn main() {
    let help_tuple = help_string();
    let pkg_name = help_tuple.0;
    let tmpl_type = help_tuple.1;
    let force_overwrite = help_tuple.2;
    let is_verbose = help_tuple.3;
    let is_debug = help_tuple.4;

    set_up_logging(is_debug, is_verbose);

    let pkg_type = figure_out_provider(tmpl_type, &pkg_name)
        .map_err(|e| err_handler(&e.to_string()))
        .unwrap();

    template_handler(&pkg_name, &pkg_type, force_overwrite);
}
