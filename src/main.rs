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
extern crate log;
#[macro_use]
extern crate clap;

use crates_io_api::SyncClient;
use clap::App;
use std::fs::File;
use std::io::prelude::Write;
use std::process::Command;
use std::str::from_utf8;

#[derive(Debug)]
struct PkgInfo{
    pkg_name: String,
    version: String,
    description: String,
    homepage: String,
    license: String,
}

fn help_string() -> (String, String ,bool) {
    let help_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(help_yaml).get_matches();

    let tmpl_type = String::from(matches.value_of("tmpltype").unwrap());

    let mut is_verbose = false;
    if matches.is_present("verbose") {
        is_verbose = true;
    }

    let crate_name = String::from(matches.value_of("INPUT").unwrap());

    (crate_name, tmpl_type, is_verbose)
}

fn crate_info(crate_name: &String) -> (PkgInfo) {
    let client = SyncClient::new();

    let query_result = client.full_crate(crate_name, false);

    if query_result.is_err() {
        error!("Failed to query crates.io for info");
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

fn write_template(pkg_info: &PkgInfo) -> Result<(), std::io::Error> {
    let mut template_in= include_str!("template.in");

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

    println!("{}", template_string);

    let mut file = File::create("template")?;

    file.write_all(template_string.as_bytes())?;

    Ok(())
}

fn main() {
    let help_tuple = help_string();
    let pkg_name = help_tuple.0;
    let tmpl_type = help_tuple.1;
    let is_verbose = help_tuple.2;

    if is_verbose {
        println!("Generating template for package {} of type {}", pkg_name, tmpl_type);
    }

    let pkg_info = crate_info(&pkg_name);

    write_template(&pkg_info).expect("Failed to write template!");
}
