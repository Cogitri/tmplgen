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

mod crates;
mod gems;
mod helpers;
#[cfg(test)]
mod tests;
mod tmplwriter;
mod types;

use clap::App;
use crates::*;
use gems::*;
use tmplwriter::*;

// Print the help script if invoked without arguments or with `--help`/`-h`
pub fn help_string() -> (String, String, bool) {
    let help_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(help_yaml).get_matches();

    let tmpl_type = String::from(matches.value_of("tmpltype").unwrap());

    let crate_name = String::from(matches.value_of("INPUT").unwrap());

    let force_overwrite = matches.is_present("force");

    (crate_name, tmpl_type, force_overwrite)
}

fn main() {
    let help_tuple = help_string();
    let pkg_name = help_tuple.0;
    let tmpl_type = help_tuple.1;
    let force_overwrite = help_tuple.2;

    println!(
        "Generating template for package {} of type {}",
        pkg_name, tmpl_type
    );

    let pkg_info = if tmpl_type == "crate" {
        crate_info(&pkg_name).expect("Failed to get the crate's info")
    } else {
        gem_info(&pkg_name).expect("Failed to get the gem's info")
    };

    write_template(&pkg_info, force_overwrite, tmpl_type).expect("Failed to write the template!");
}
