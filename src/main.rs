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

use clap::App;
use std::fs::{create_dir_all, File};
use std::io::prelude::Write;
use std::path::Path;
use std::process::{exit, Command};
use std::str::from_utf8;

#[derive(Fail, Debug)]
enum Error {
    #[fail(display = "{}", _0)]
    File(std::io::Error),
    #[fail(display = "{}", _0)]
    Crate(crates_io_api::Error),
    #[fail(display = "{}", _0)]
    Gem(rubygems_api::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::File(e)
    }
}

impl From<crates_io_api::Error> for Error {
    fn from(e: crates_io_api::Error) -> Self {
        Error::Crate(e)
    }
}

impl From<rubygems_api::Error> for Error {
    fn from(e: rubygems_api::Error) -> Self {
        Error::Gem(e)
    }
}

struct Dependencies {
    make: Vec<String>,
    run: Vec<String>,
}

struct PkgInfo {
    pkg_name: String,
    version: String,
    description: String,
    homepage: String,
    license: Vec<String>,
    dependencies: Option<Dependencies>,
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
fn crate_info(crate_name: &String) -> Result<PkgInfo, Error> {
    let client = crates_io_api::SyncClient::new();

    let query_result = client.full_crate(crate_name, false)?;

    let pkg_info = PkgInfo {
        pkg_name: crate_name.clone(),
        version: query_result.max_version,
        description: query_result
            .description
            .unwrap_or_else(|| missing_field_s("description")),
        homepage: query_result
            .homepage
            .unwrap_or_else(|| missing_field_s("homepage")),
        license: vec![
            query_result
                .license
                .unwrap_or_else(|| missing_field_s("license")),
        ],
        dependencies: None,
    };

    Ok(pkg_info)
}

fn gem_info(gem_name: &String) -> Result<PkgInfo, Error> {
    let client = rubygems_api::SyncClient::new();

    let query_result = client.gem_info(gem_name)?;

    let mut dep_vec_dev = Vec::new();

    for x in query_result.dependencies.development.unwrap() {
        let dep = determine_gem_dev_deps(x)?;
        dep_vec_dev.push(dep.unwrap());
    }

    let mut dep_vec_run = Vec::new();
    for x in query_result.dependencies.runtime.unwrap() {
        let dep = determine_gem_run_deps(x)?;
        dep_vec_run.push(dep.unwrap());
    }

    let pkg_info = PkgInfo {
        pkg_name: gem_name.clone(),
        version: query_result.version,
        description: query_result
            .info
            .unwrap_or_else(|| missing_field_s("description")),
        homepage: query_result
            .homepage_uri
            .unwrap_or_else(|| missing_field_s("homepage")),
        license: query_result
            .licenses
            .unwrap_or_else(|| missing_field_v("license")),
        dependencies: Some(Dependencies {
            make: dep_vec_dev,
            run: dep_vec_run,
        }),
    };

    Ok(pkg_info)
}

fn tilde_parse(version: String) -> Option<Vec<String>> {
    let ver_vec = version.split(".").collect::<Vec<_>>();

    match ver_vec.len() {
        1 => Some(vec![
            String::from(">=".to_owned() + &version),
            String::from("<".to_owned() + &(ver_vec[0].parse::<u8>().unwrap() + 1).to_string()),
        ]),
        2 => Some(vec![
            String::from(">=".to_owned() + &version),
            String::from("<".to_owned() + &(ver_vec[0].parse::<u8>().unwrap() + 1).to_string()),
        ]),
        3 => Some(vec![
            String::from(">=".to_owned() + &version),
            String::from(
                "<".to_owned()
                    + &ver_vec[0]
                    + &".".to_owned()
                    + &(ver_vec[1].parse::<u8>().unwrap() + 1).to_string(),
            ),
        ]),
        _ => None,
    }
}

fn determine_gem_dev_deps(rubygem_dep: rubygems_api::GemDevDeps) -> Result<Option<String>, Error> {
    let cmpr = String::from(
        rubygem_dep
            .requirements
            .split_whitespace()
            .collect::<Vec<_>>()[0],
    );
    let ver = String::from(
        rubygem_dep
            .requirements
            .split_whitespace()
            .collect::<Vec<_>>()[1],
    );

    let ver_req = match cmpr.as_ref() {
        ">" | "<" | "<=" => Some(rubygem_dep.name + &cmpr + &ver),
        ">=" => if ver == "0" {
            Some(rubygem_dep.name)
        } else {
            Some(rubygem_dep.name + &cmpr + &ver)
        },
        "~>" => {
            let tilde_vec = tilde_parse(ver).unwrap();
            Some(
                "".to_string()
                    + &rubygem_dep.name
                    + &tilde_vec[0]
                    + &" ".to_string()
                    + &rubygem_dep.name
                    + &tilde_vec[1]
                    + &" ".to_string(),
            )
        }
        _ => None,
    };

    Ok(ver_req)
}

fn determine_gem_run_deps(rubygem_dep: rubygems_api::GemRunDeps) -> Result<Option<String>, Error> {
    let cmpr = String::from(
        rubygem_dep
            .requirements
            .split_whitespace()
            .collect::<Vec<_>>()[0],
    );
    let ver = String::from(
        rubygem_dep
            .requirements
            .split_whitespace()
            .collect::<Vec<_>>()[1],
    );

    let ver_req = match cmpr.as_ref() {
        ">" | "<" | "<=" => Some(rubygem_dep.name + &cmpr + &ver),
        ">=" => if ver == "0" {
            Some(rubygem_dep.name)
        } else {
            Some(rubygem_dep.name + &cmpr + &ver)
        },
        "~>" => {
            let tilde_vec = tilde_parse(ver).unwrap();
            Some(
                "".to_string()
                    + &rubygem_dep.name
                    + &tilde_vec[0]
                    + &" ".to_string()
                    + &rubygem_dep.name
                    + &tilde_vec[1]
                    + &" ".to_string(),
            )
        }
        _ => None,
    };

    Ok(ver_req)
}

//fn missing_field<T>(field_name: &str) -> T {
//    eprintln!("Couldn't determine field '{}'! Please add it to the template yourself.", field_name);
//
//    T::new()
//}

fn missing_field_s(field_name: &str) -> String {
    eprintln!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    String::from("")
}

fn missing_field_v(field_name: &str) -> Vec<String> {
    eprintln!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    vec![String::from("")]
}

// Writes the PkgInfo to a file called "template"
fn write_template(
    pkg_info: &PkgInfo,
    force_overwrite: bool,
    tmpl_type: String,
) -> Result<(), Error> {
    let template_in = include_str!("template.in");

    let git_author = Command::new("git")
        .args(&["config", "user.name"])
        .output()
        .expect("Couldn't determine git username!");
    let git_mail = Command::new("git")
        .args(&["config", "user.email"])
        .output()
        .expect("Couldn't determine git username!");

    let mut maintainer = format!(
        "{} <{}>",
        from_utf8(&git_author.stdout).expect("Failed to decode git author!"),
        from_utf8(&git_mail.stdout).expect("Failed to decode git email!"),
    );
    maintainer = maintainer.replace("\n", "");

    let mut template_string = template_in
        .replace("@version@", &pkg_info.version)
        .replace("@description@", &pkg_info.description)
        .replace("@license@", &pkg_info.license.join(","))
        .replace("@homepage@", &pkg_info.homepage)
        .replace("@maintainer@", &maintainer);

    let dependencies = &pkg_info.dependencies.as_ref().unwrap();

    let mut makedepends = String::new();
    let mut depends = String::new();

    for x in &dependencies.make {
        makedepends.push_str(x);
    }
    for x in &dependencies.run {
        depends.push_str(x);
    }

    if tmpl_type == "gem" {
        if pkg_info.dependencies.is_some() {
            if &dependencies.make.len() != &0 {
                template_string = template_string.replace("@makedepends@", &makedepends.trim_end())
            } else {
                template_string = template_string.replace("\nmakedepends=\"@makedepends\"", "")
            }

            if &dependencies.run.len() != &0 {
                template_string = template_string.replace("@depends@", &depends.trim_end())
            } else {
                template_string = template_string.replace("\ndepends=\"@depends@\"", "")
            }

            template_string = template_string
                .replace(
                    "@pkgname@",
                    &String::from("ruby-".to_owned() + &pkg_info.pkg_name),
                ).replace("@build_style@", "gem")
                .replace("\ndistfiles=\"@distfiles@\"", "");
        }
    } else {
        template_string = template_string
            .replace("@pkgname@", &pkg_info.pkg_name)
            .replace("\nmakedepends=\"@makedepends\"", "")
            .replace("\ndepends=\"@depends@\"", "")
            .replace("@build_style@", "cargo")
            .replace(
                "@distfiles@",
                &format!(
                    "https://static.crates.io/crates/{name}/{name}-${{version}}.crate",
                    name = &pkg_info.pkg_name
                ),
            );
    }

    let xdistdir = Command::new("sh")
        .args(&["-c", "xdistdir"])
        .output()
        .expect("Couldn't execute xdistdir. Make sure you have xtools installed.");

    let xbps_distdir = format!(
        "{}/srcpkgs/{}",
        from_utf8(&xdistdir.stdout)
            .unwrap()
            .replace("\n", "")
            .replace(
                "~",
                &std::env::var("HOME")
                    .expect("Please either replace '~' with your homepath or export HOME")
            ),
        &pkg_info.pkg_name
    );

    if !xdistdir.status.success() {
        println!(
            "xdistdir: exited with a non-0 exit code:\n{}",
            from_utf8(&xdistdir.stderr).unwrap()
        );
    }

    if Path::new(&format!("{}/template", &xbps_distdir)).exists() && !force_overwrite {
        eprintln!(
            "Won't overwrite existing template '{}/template' without `--force`!",
            &xbps_distdir
        );
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
