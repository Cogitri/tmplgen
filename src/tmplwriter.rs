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
use std::io::prelude::*;
use std::path::Path;
use types::*;

// Writes the PkgInfo to a file called "template"
pub fn write_template(
    pkg_info: &PkgInfo,
    force_overwrite: bool,
    tmpl_type: &PkgType,
) -> Result<(), Error> {
    let template_in = include_str!("template.in");

    let maintainer= get_git_author()?;

    let mut license = String::new();

    for x in &pkg_info.license {
        if ! license.is_empty() {
            license.push_str(", ");
        }

        license.push_str(&correct_license(x));
    }

    let mut template_string = template_in
        .replace("@version@", &pkg_info.version)
        .replace(
            "@description@",
            &check_string_len(&pkg_info.pkg_name, &pkg_info.description, "description"),
        )
        .replace("@license@", &license)
        .replace("@homepage@", &pkg_info.homepage)
        .replace("@maintainer@", &maintainer)
        .replace("@pkgname@", &pkg_info.pkg_name)
        .replace("@checksum@", &pkg_info.sha);

    if pkg_info.dependencies.is_some() {
        let dependencies = pkg_info.dependencies.as_ref().unwrap();

        if dependencies.host.is_some() {
            let host_depends = gen_dep_string(dependencies.host.as_ref().unwrap(), tmpl_type);
            template_string =
                template_string.replace("@hostmakedepends@", &host_depends.trim_end());
        } else {
            template_string =
                template_string.replace("\nhostmakedepends=\"@hostmakedepends@\"", "");
        }
        if dependencies.make.is_some() {
            let make_depends = gen_dep_string(dependencies.make.as_ref().unwrap(), tmpl_type);
            template_string = template_string.replace("@makedepends@", &make_depends.trim_end());
        } else {
            template_string = template_string.replace("\nmakedepends=\"@makedepends@\"", "");
        }
        if dependencies.run.is_some() {
            let run_depends = gen_dep_string(dependencies.run.as_ref().unwrap(), tmpl_type);
            template_string = template_string.replace("@depends@", &run_depends.trim_end());
        } else {
            template_string = template_string.replace("\ndepends=\"@depends@\"", "");
        }
    } else {
        template_string = template_string.replace("\ndepends=\"@depends@\"", "");
        template_string = template_string.replace("\nmakedepends=\"@makedepends@\"", "");
        template_string = template_string.replace("\nhostmakedepends=\"@hostmakedepends@\"", "");
    }

    if pkg_info.download_url.is_some() {
        template_string =
            template_string.replace("@distfiles@", &pkg_info.download_url.as_ref().unwrap());

    } else {
        template_string = template_string.replace("\ndistfiles=\"@distfiles@\"", "")
    }

    if tmpl_type == &PkgType::PerlDist {
        template_string = template_string
            .replace("@build_style@", "perl-module")
            .replace("@noarch@", "yes")
            .replace("@wrksrc@", "${pkgname/perl-/}-${version}");
    } else if tmpl_type == &PkgType::Gem {
        template_string = template_string
            .replace("@build_style@", "gem")
            .replace("\nwrksrc=\"@wrksrc@\"", "")
            .replace("\nnoarch=@noarch@", "");
    } else {
        template_string = template_string
            .replace("@pkgname@", &pkg_info.pkg_name)
            .replace("\ndepends=\"@depends@\"", "")
            .replace("@build_style@", "cargo")
            .replace("\nwrksrc=\"@wrksrc@\"", "")
            .replace("\nnoarch=@noarch@", "");
    }

    let license = &pkg_info.license.join(", ");
    if license.contains(&"MIT".to_string())
        || license.contains(&"ISC".to_string())
        || license.contains(&"BSD".to_string())
    {
        template_string.push_str("\n\npost_install() {\n\tvlicense LICENSE\n}");
    }

    template_string.push_str("\n");

    let xdist_template_path = format!("{}{}", xdist_files()?, &pkg_info.pkg_name);

    if Path::new(&format!("{}/template", &xdist_template_path)).exists() && !force_overwrite {
        return Err(Error::TmplWriter(format!(
            "Won't overwrite existing template '{}/template' without `--force`!",
            &xdist_template_path,
        )));
    }

    info!("Writing template to path {}/template", &xdist_template_path);

    create_dir_all(&xdist_template_path)?;
    let mut file = File::create(format!("{}/template", &xdist_template_path))?;

    file.write_all(template_string.as_bytes())?;

    Ok(())
}

pub fn update_template(pkg_info: &PkgInfo, update_all: bool, force_overwrite: bool) -> Result<(), Error> {
    info!("Updating template {}", &pkg_info.pkg_name);

    let xdist_template = format!("{}{}/template", xdist_files()?, &pkg_info.pkg_name);

    if Path::new(&xdist_template).exists() {
        let mut template_file = File::open(&xdist_template)?;
        let mut template_string = String::new();
        template_file.read_to_string(&mut template_string)?;

        let mut orig_ver_string = String::new();
        let mut orig_checksum_string = String::new();
        let mut orig_distfiles_string = String::new();

        for x in template_string.lines() {
            if x.contains("version=") {
                orig_ver_string = x.to_string();
            }
            if x.contains("checksum=") {
                orig_checksum_string = x.to_string();
            }
        }

        if &pkg_info.version == &orig_ver_string.replace("version=", "") {
            if force_overwrite {
                warn!("Updating already up-to-date template");
            } else {
                return Err(Error::TmplUpdater("Template is already up-to-date, refusing to overwrite it without `--force`!".to_string()));
            }
        }

        let mut orig_homepage_string = String::new();
        let mut orig_description_string = String::new();

        if update_all {
            for x in template_string.lines() {
                if x.contains("homepage=") {
                    orig_homepage_string = x.to_string();
                }
                if x.contains("short_desc=") {
                    orig_description_string = x.to_string();
                }
                if x.contains("distfiles=") {
                    orig_distfiles_string = x.to_string();
                }
            }
        }

        template_string = template_string.replace(&orig_ver_string, &format!("version={}", &pkg_info.version));

        if update_all {
            template_string = template_string.replace(&orig_checksum_string, &format!("checksum={}", &pkg_info.sha));

            if orig_homepage_string.is_empty() {
                warn!("Couldn't find 'homepage' string and as such won't update it!");
            } else {
                template_string = template_string.replace(&orig_homepage_string, &format!("homepage=\"{}\"", &pkg_info.homepage));
            }

            if orig_distfiles_string.is_empty() {
                warn!("Couldn't find 'distfiles' string and as such won't update it!");
            } else {
                // This looks a bit funny because...well, it is. download_url can be empty, but we
                // want to just remove the previous distfiles, in case the gem downloads some additional
                // data
                template_string = template_string.replace(&orig_distfiles_string, &format!("distfiles=\"{}", format!("{}\"", &pkg_info.download_url.as_ref().unwrap_or(&orig_distfiles_string.replace("distfiles=\"", "")))));
            }

            if orig_description_string.is_empty() {
                warn!("Couldn't find 'description' string and as such won't update it!");
            } else {
                template_string = template_string.replace(&orig_description_string, &format!("short_desc=\"{}\"", &pkg_info.description));
            }
        }

        let mut template_file = File::create(&xdist_template)?;
        template_file.write_all(template_string.as_bytes())?;

    } else {
        return Err(Error::TmplUpdater("Template doesn't exist yet; can't update a non-existant template!".to_string()));
    }

    Ok(())
}