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

use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    let mut cli_yml_in = OpenOptions::new()
        .read(true)
        .open("src/bin/cli.yml")
        .unwrap();

    let mut cli_string = String::new();
    cli_yml_in.read_to_string(&mut cli_string).unwrap();

    cli_string = cli_string.replace("@version_string@", version);

    let mut cli_yml_out = OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open("src/bin/cli-build.yml")
        .unwrap();

    cli_yml_out.write_all(cli_string.as_bytes()).unwrap();
}
