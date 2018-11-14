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

use helpers::*;
use std::fs::{create_dir_all, File};
use std::io::prelude::Write;
use std::path::Path;
use std::process::{exit, Command};
use std::str::from_utf8;
use types::*;

// Writes the PkgInfo to a file called "template"
pub fn write_template(
    pkg_info: &PkgInfo,
    force_overwrite: bool,
    tmpl_type: &PkgType,
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
        .replace("@description@", &check_string_len(&pkg_info.description, "description"))
        .replace("@license@", &pkg_info.license.join(", "))
        .replace("@homepage@", &pkg_info.homepage)
        .replace("@maintainer@", &maintainer);

    if tmpl_type == &PkgType::Gem {
        let dependencies = &pkg_info.dependencies.as_ref().unwrap();

        let mut depends = String::new();

        for x in dependencies.run.as_ref().unwrap() {
            depends.push_str(x);
            if depends.len() >= 80 {
                depends.push_str("\\n");
            }
        }

        if pkg_info.dependencies.is_some() {
            if &dependencies.run.as_ref().unwrap().len() != &0 {
                template_string = template_string.replace("@depends@", &depends.trim_end())
            } else {
                template_string = template_string.replace("\ndepends=\"@depends@\"", "")
            }

            template_string = template_string
                .replace("@pkgname@", &pkg_info.pkg_name)
                .replace("@build_style@", "gem")
                .replace("\ndistfiles=\"@distfiles@\"", "");
        }
    } else {
        template_string = template_string
            .replace("@pkgname@", &pkg_info.pkg_name)
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

    let license = &pkg_info.license.join(", ");
    if license.contains(&"MIT".to_string()) || license.contains(&"ISC".to_string()) || license.contains(&"BSD".to_string()) {
        template_string.push_str("\n\npost_install() {\n\tvlicense LICENSE\n}");
    }

    template_string.push_str("\n");

    let xdist_template_path = format!("{}{}", xdist_files(), &pkg_info.pkg_name);

    if Path::new(&format!("{}/template", &xdist_template_path)).exists() && !force_overwrite {
        error!(
            "Won't overwrite existing template '{}/template' without `--force`!",
            &xdist_template_path,
        );
        exit(1);
    }

    info!("Writing template to path {}/template", &xdist_template_path);

    create_dir_all(format!("{}", &xdist_template_path))?;
    let mut file = File::create(format!("{}/template", &xdist_template_path))?;

    file.write_all(template_string.as_bytes())?;

    Ok(())
}
