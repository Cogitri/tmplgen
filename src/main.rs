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

use crates_io_api::SyncClient;
use clap::App;
use std::fs::{create_dir_all,File};
use std::io::prelude::Write;
use std::path::Path;
use std::process::{Command, exit};
use std::str::from_utf8;

struct PkgInfo{
    pkg_name: String,
    version: String,
    description: String,
    homepage: String,
    license: String,
}

// Print the help script if invoked without arguments or with `--help`/`-h`
fn help_string() -> (String, String, bool) {
    let help_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(help_yaml).get_matches();

    let tmpl_type = String::from(matches.value_of("tmpltype").unwrap());

    let crate_name = String::from(matches.value_of("INPUT").unwrap());

    let force_overwrite = matches.is_present("force");

    (crate_name, tmpl_type, force_overwrite)
}

// Query the crates.io API. Returns a PkgInfo that contains all important info
fn crate_info(crate_name: &String) -> (PkgInfo) {
    let client = SyncClient::new();

    let query_result = client.full_crate(crate_name, false);

    if query_result.is_err() {
        eprintln!("Failed to query crates.io for info");
    }

    let crate_opt = query_result.ok();

    let crate_obj = crate_opt.unwrap();

    PkgInfo {
        pkg_name: crate_name.clone(),
        version: crate_obj.max_version,
        description: crate_obj.description.unwrap(),
        homepage: crate_obj.homepage.unwrap(),
        license: crate_obj.license.unwrap_or_default(),
    }
}

// Writes the PkgInfo to a file called "template"
fn write_template(pkg_info: &PkgInfo, force_overwrite: bool) -> Result<(), std::io::Error> {
    let template_in = include_str!("template.in");

    let git_author = Command::new("git").args(&["config", "user.name"]).output().expect("Couldn't determine git username!");
    let git_mail = Command::new("git").args(&["config", "user.email"]).output().expect("Couldn't determine git username!");

    let mut maintainer = format!("{} <{}>", from_utf8(&git_author.stdout).unwrap(), from_utf8(&git_mail.stdout).unwrap());
    maintainer = maintainer.replace("\n", "");

    let mut template_string = template_in.replace("@pkgname@", &pkg_info.pkg_name);
    template_string = template_string.replace("@version@", &pkg_info.version);
    template_string = template_string.replace("@build_style@", "cargo");
    template_string = template_string.replace("@description@", &pkg_info.description);
    template_string = template_string.replace("@license@", &pkg_info.license);
    template_string = template_string.replace("@homepage@", &pkg_info.homepage);
    template_string = template_string.replace("@maintainer@", &maintainer);
    template_string = template_string.replace("@distfiles@", &format!("https://static.crates.io/crates/{name}/{name}-${{version}}.crate", name = &pkg_info.pkg_name));

    let xdistdir = Command::new("sh").args(&["-c", "xdistdir"]).output().expect("Couldn't execute xdistdir. Make sure you have xtools installed.");

    let xbps_distdir = format!("{}/srcpkgs/{}", from_utf8(&xdistdir.stdout).unwrap().replace("\n", "").replace("~", &std::env::var("HOME").expect("Please either replace '~' with your homepath or export HOME")), &pkg_info.pkg_name);

    if ! xdistdir.status.success() {
        println!("xdistdir: exited with a non-0 exit code:\n{}", from_utf8(&xdistdir.stderr).unwrap());
    }

    if Path::new(&format!("{}/template", &xbps_distdir)).exists() && !force_overwrite {
        eprintln!("Won't overwrite existing template '{}/template' without `--force`!", &xbps_distdir);
        exit(1);
    }

    println!("Writing template to path {}/template", &xbps_distdir);

    create_dir_all(&xbps_distdir)?;
    let mut file = File::create(format!("{}/template", &xbps_distdir))?;

    file.write_all(template_string.as_bytes())?;

    Ok(())
}

fn main() {
    let help_tuple = help_string();
    let pkg_name = help_tuple.0;
    let tmpl_type = help_tuple.1;
    let force_overwite = help_tuple.2;

    println!("Generating template for package {} of type {}", pkg_name, tmpl_type);

    let pkg_info = crate_info(&pkg_name);

    write_template(&pkg_info, force_overwite).expect("Failed to write template!");
}
