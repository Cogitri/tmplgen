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

    if tmpl_type == "gem" {
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
                    &String::from("ruby-".to_owned() + &pkg_info.pkg_name),
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
