use std::fs::{create_dir_all, File};
use std::io::prelude::Write;
use std::process::{exit,Command};
use std::path::Path;
use std::str::from_utf8;
use types::*;
use helpers::*;

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
        .replace("@description@", &pkg_info.description)
        .replace("@license@", &pkg_info.license.join(","))
        .replace("@homepage@", &pkg_info.homepage)
        .replace("@maintainer@", &maintainer);

    if tmpl_type == &PkgType::Gem {
        let dependencies = &pkg_info.dependencies.as_ref().unwrap();

        let mut makedepends = String::new();
        let mut depends = String::new();

        for x in &dependencies.make {
            makedepends.push_str(x);
        }
        for x in &dependencies.run {
            depends.push_str(x);
        }

        if pkg_info.dependencies.is_some() {
            if &dependencies.make.len() != &0 {
                template_string = template_string.replace("@makedepends@", &makedepends.trim_end())
            } else {
                template_string = template_string.replace("\nmakedepends=\"@makedepends@\"", "")
            }

            if &dependencies.run.len() != &0 {
                template_string = template_string.replace("@depends@", &depends.trim_end())
            } else {
                template_string = template_string.replace("\ndepends=\"@depends@\"", "")
            }

            template_string = template_string
                .replace(
                    "@pkgname@",
                    &pkg_info.pkg_name,
                ).replace("@build_style@", "gem")
                .replace("\ndistfiles=\"@distfiles@\"", "");
        }
    } else {
        template_string = template_string
            .replace("@pkgname@", &pkg_info.pkg_name)
            .replace("\nmakedepends=\"@makedepends@\"", "")
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

    let xbps_distdir = xdist_files();

    if Path::new(&format!("{}/{}/template", &xbps_distdir, &pkg_info.pkg_name)).exists() && !force_overwrite {
        error!(
            "Won't overwrite existing template '{}{}/template' without `--force`!",
            &xbps_distdir,
            &pkg_info.pkg_name
        );
        exit(1);
    }

    info!("Writing template to path {}{}/template", &xbps_distdir, &pkg_info.pkg_name);

    create_dir_all(format!("{}{}", &xbps_distdir, &pkg_info.pkg_name))?;
    let mut file = File::create(format!("{}{}/template", &xbps_distdir, &pkg_info.pkg_name))?;

    file.write_all(template_string.as_bytes())?;

    Ok(())
}
