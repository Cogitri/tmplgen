use crates::*;
use gems::*;
use std::path::Path;
use std::process::{exit, Command};
use std::str::from_utf8;
use tmplwriter::*;
use types::*;

pub fn missing_field_s(field_name: &str) -> String {
    warn!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    String::from("")
}

pub fn missing_field_v(field_name: &str) -> Vec<String> {
    warn!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    vec![String::from("")]
}

pub fn figure_out_provider(
    tmpl_type: Option<PkgType>,
    pkg_name: &String,
) -> Result<PkgType, String> {
    if tmpl_type.is_none() {
        let crate_status = crates_io_api::SyncClient::new()
            .get_crate(&pkg_name)
            .is_ok();

        let gem_status = rubygems_api::SyncClient::new().gem_info(&pkg_name).is_ok();

        if crate_status && gem_status {
            Err("Found a package with the specified name both on crates.io and rubygems.org! Please explicitly choose one via the `-t` parameter!".to_string())
        } else if crate_status {
            debug!("Determined the target package to be a crate");
            Ok(PkgType::Crate)
        } else if gem_status {
            debug!("Determined the target package to be a ruby gem");
            Ok(PkgType::Gem)
        } else {
            Err("Unable to determine what type of the target package! Make sure you've spelled the package name correctly!".to_string())
        }
    } else {
        Ok(tmpl_type.unwrap())
    }
}

pub fn template_handler(pkg_name: String, pkg_type: &PkgType, force_overwrite: bool) {
    info!(
        "Generating template for package {} of type {:?}",
        &pkg_name, pkg_type
    );

    let pkg_info = if pkg_type == &PkgType::Crate {
        crate_info(&pkg_name).expect("Failed to get the crate's info")
    } else {
        gem_info(&pkg_name).expect("Failed to get the gem's info")
    };

    write_template(&pkg_info, force_overwrite, &pkg_type).expect("Failed to write the template!");

    if pkg_type == &PkgType::Gem {
        gem_dep_graph(&pkg_name, force_overwrite);
    }
}

pub fn xdist_files() -> String {
    let xdistdir = Command::new("sh")
        .args(&["-c", "xdistdir"])
        .output()
        .expect("Couldn't execute xdistdir. Make sure you have xtools installed.");

    if !xdistdir.status.success() {
        error!(
            "xdistdir: exited with a non-0 exit code:\n {:?}",
            from_utf8(&xdistdir.stderr).unwrap()
        );

        exit(1);
    }

    format!(
        "{}/srcpkgs/",
        from_utf8(&xdistdir.stdout)
            .unwrap()
            .replace("\n", "")
            .replace(
                "~",
                &std::env::var("HOME")
                    .expect("Please either replace '~' with your homepath or export HOME")
            ),
    )
}

pub fn recursive_deps(
    deps: &Vec<String>,
    xdistdir: &String,
    pkg_type: PkgType,
    force_overwrite: bool,
) {
    if force_overwrite {
        for x in deps {
            info!("Specified `-f`, will overwrite existing templates if they exists...");
            template_handler(x.to_string(), &pkg_type, force_overwrite);
        }
    } else {
        for x in deps {
            let tmpl_path = if pkg_type == PkgType::Gem {
                format!("{}ruby-{}/template", xdistdir, x)
            } else {
                format!("{}{}/template", xdistdir, x)
            };
            if !Path::new(&tmpl_path).exists() {
                info!(
                    "Dependency {} doesn't exist yet, writing a template for it...",
                    x
                );
                template_handler(x.to_string(), &pkg_type, force_overwrite);
            } else {
                debug!("Dependency {} is already satisfied!", x);
            }
        }
    }
}

pub fn check_string_len(string: &String, string_type: &str) -> String {
    if string.len() >= 80 {
        warn!("{} is longer than 80 characters, please cut as you see fit!", string_type);
    }

    string.to_string()
}