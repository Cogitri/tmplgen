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

use crate::crates::*;
use crate::gems::*;
use crate::perldist::*;
use crate::tmplwriter::*;
use crate::types::*;
use log::{debug, error, info, warn};
use sha2::{Digest, Sha256};
use std::env::var_os;
use std::path::Path;
use std::process::{exit, Command};
use std::str::from_utf8;

pub fn missing_field_s(field_name: &str) -> String {
    warn!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    String::from("")
}

// Figure out whether we're dealing with a crate or a gem if the user hasn't specified that.
// Errors out of a package with the name the user gave us can be found on both crates.io and
// rubygems.org
pub fn figure_out_provider(tmpl_type: Option<PkgType>, pkg_name: &str) -> Result<PkgType, Error> {
    if tmpl_type.is_none() {
        let crate_status = crates_io_api::SyncClient::new()
            .get_crate(&pkg_name)
            .is_ok();

        let gem_status = rubygems_api::SyncClient::new().gem_info(&pkg_name).is_ok();

        let perldist_status = metacpan_api::SyncClient::new().perl_info(&pkg_name).is_ok();

        if (crate_status && gem_status)
            || (crate_status && perldist_status)
            || (gem_status && perldist_status)
        {
            Err(Error::AmbPkg(pkg_name.to_string()))
        } else if crate_status {
            debug!("Determined the target package {} to be a crate", &pkg_name);
            Ok(PkgType::Crate)
        } else if gem_status {
            debug!(
                "Determined the target package {} to be a ruby gem",
                &pkg_name
            );
            Ok(PkgType::Gem)
        } else if perldist_status {
            debug!("Determined the target package to be a perldist");
            Ok(PkgType::PerlDist)
        } else {
            Err(Error::NoSuchPkg(pkg_name.to_string()))
        }
    } else {
        Ok(tmpl_type.unwrap())
    }
}

// Handle getting the necessary info and writing a template for it. Invoked every time a template
// should be written, useful for recursive deps.
pub fn template_handler(
    pkg_info: &PkgInfo,
    pkg_type: &PkgType,
    force_overwrite: bool,
    is_rec: bool,
) -> Result<(), Error> {
    let pkg_name = &pkg_info.pkg_name;

    info!(
        "Generating template for package {} of type {:?}",
        &pkg_name, pkg_type
    );

    if is_rec {
        write_template(&pkg_info, force_overwrite, &pkg_type)
            .map_err(|e| warn!("Failed to write the template for dep {}: {}", pkg_name, e))
            .unwrap_or_default()
    } else {
        write_template(&pkg_info, force_overwrite, &pkg_type)?;
    }

    if pkg_type == &PkgType::Crate {
        return Ok(());
    }

    let dep_graph = if pkg_type == &PkgType::Gem {
        gem_dep_graph(&pkg_name.replace("ruby-", ""))
    } else {
        perldist_dep_graph(&pkg_name.replace("perl-", ""))
    };

    if dep_graph.is_err() {
        warn!(
            "Failed to write templates for all recursive deps of {}! Error: {}",
            pkg_name,
            dep_graph.unwrap_err()
        );
    }

    Ok(())
}

// Figure out where to write template files with `xdistdir`
pub fn xdist_files() -> Result<String, Error> {
    let xdistdir = Command::new("sh").args(&["-c", "xdistdir"]).output()?;

    if !xdistdir.status.success() {
        return Err(Error::XdistError(
            from_utf8(&xdistdir.stderr).unwrap().to_string(),
        ));
    }

    let xdistdir_string = from_utf8(&xdistdir.stdout)?;

    if xdistdir_string.contains('~') {
        let home_dir = std::env::var("HOME");

        if home_dir.is_err() {
            return Err(Error::XdistError(
                "Please either replace '~' with your homepath in XBPS_XDISTDIR or export HOME"
                    .to_string(),
            ));
        }

        let xdistdir_path = &from_utf8(&xdistdir.stdout)?.replace("~", &home_dir.ok().unwrap());

        Ok(format!("{}/srcpkgs/", xdistdir_path.replace("\n", ""),))
    } else {
        Ok(format!("{}/srcpkgs/", xdistdir_string.replace("\n", ""),))
    }
}

// Generic function to handle recursive deps.
pub fn recursive_deps(deps: &[String], xdistdir: &str, pkg_type: &PkgType) -> Result<(), Error> {
    for x in deps {
        // We want to ignore built-in deps
        if !is_built_in(x, pkg_type) {
            let tmpl_path = if pkg_type == &PkgType::Gem {
                format!("{}ruby-{}/template", xdistdir, x)
            } else if pkg_type == &PkgType::PerlDist {
                // We don't write templates for modules, but only
                // for distributions. As such we have to convert
                // the module's name to the distribution's name,
                // if we're handling a module
                let perl_client = metacpan_api::SyncClient::new();

                let dist = perl_client.get_dist(&x);

                if dist.is_ok() {
                    format!(
                        "{}perl-{}/template",
                        xdistdir,
                        dist.unwrap().replace("::", "-")
                    )
                } else {
                    format!("{}perl-{}/template", xdistdir, x.replace("::", "-"))
                }
            } else {
                format!("{}{}/template", xdistdir, x)
            };

            debug!("Checking for template in {}...", &tmpl_path);

            if !Path::new(&tmpl_path).exists() {
                info!(
                    "Dependency {} doesn't exist yet, writing a template for it...",
                    x
                );
                template_handler(&get_pkginfo(&x, pkg_type)?, &pkg_type, false, true)?;
            } else {
                debug!("Dependency {} is already satisfied!", x);
            }
        }
    }
    Ok(())
}

pub fn check_string_len(pkg_name: &str, string: &str, string_type: &str) -> String {
    if string.len() >= 80 {
        warn!(
            "{} of package {} is longer than 80 characters, please cut as you see fit!",
            pkg_name, string_type
        );
    }

    string.to_string()
}

pub fn is_built_in(pkg_name: &str, pkg_type: &PkgType) -> bool {
    if pkg_type == &PkgType::Crate {
        return false;
    }

    let data: BuiltIns = serde_json::from_str(include_str!("built_in.in")).unwrap();

    let built_ins = BuiltIns {
        perl: data.perl,
        ruby: data.ruby,
    };

    if pkg_type == &PkgType::Gem {
        for x in built_ins.ruby {
            if pkg_name == x.name {
                warn!(
                    "Gem {} is part of ruby, won't write a template for it!",
                    pkg_name
                );
                return true;
            }
        }
    } else if pkg_type == &PkgType::PerlDist {
        let pkg_name = pkg_name.replace("::", "-");

        for x in built_ins.perl {
            if pkg_name == x.name {
                warn!(
                    "Perl distribution {} is part of perl, won't write a template for it!",
                    pkg_name
                );
                return true;
            }
        }
    }

    false
}

pub fn gen_dep_string(dep_vec: &[String], pkg_type: &PkgType) -> String {
    let mut dep_string = String::new();

    for x in dep_vec {
        let after_string = "".to_string() + &dep_string + x;

        let last_line_ln = after_string.lines().last().unwrap_or_default().len();

        // If the string with the new dep added _plus_ the {make,host,}depends=""
        // is longer than 80 chars, we want to split the line and insert a leading
        // space to the new line.
        // Otherwise, we want to add a space to the string (to seperate two deps),
        // but we don't want to introduce leading whitespace
        if last_line_ln >= 65 {
            dep_string.push_str("\n ");
        } else if last_line_ln > x.len() {
            dep_string.push_str(" ");
        }

        if pkg_type == &PkgType::PerlDist {
            if x == "perl" {
            } else {
                dep_string.push_str(&("perl-".to_string() + &x.replace("::", "-")));
            }
        } else {
            dep_string.push_str(x);
        }
    }

    dep_string
}

// TODO: Doing it this way means that all error using this function will show up as "ERROR tmplgen::helpers" in env_logger
pub fn err_handler(error: &Error) {
    error!("{}", error.to_string());
    exit(1);
}

pub fn correct_license(license: &str) -> String {
    let data: CorrectedVals = serde_json::from_str(include_str!("corrected_values.in")).unwrap();

    let corrected_vals = CorrectedVals {
        licenses: data.licenses,
    };

    for x in corrected_vals.licenses {
        if license == x.is {
            return x.should.to_string();
        }
    }

    license.to_string()
}

pub fn get_pkginfo(pkg_name: &str, pkg_type: &PkgType) -> Result<PkgInfo, Error> {
    if pkg_type == &PkgType::Crate {
        crate_info(&pkg_name)
    } else if pkg_type == &PkgType::PerlDist {
        perldist_info(&pkg_name)
    } else {
        gem_info(pkg_name)
    }
}

pub fn get_git_author() -> Result<String, Error> {
    let git_author_env = var_os("GIT_AUTHOR_NAME");
    let git_email_env = var_os("GIT_AUTHOR_EMAIL");

    let git_details = if git_author_env.is_some() && git_email_env.is_some() {
        (
            git_author_env.unwrap().to_str().unwrap().to_string(),
            git_email_env.unwrap().to_str().unwrap().to_string(),
        )
    } else {
        let git_author = Command::new("git")
            .args(&["config", "user.name"])
            .output()?;
        let git_mail = Command::new("git")
            .args(&["config", "user.email"])
            .output()?;

        if !git_author.status.success() {
            return Err(Error::GitError(from_utf8(&git_author.stderr)?.to_string()));
        }

        if !git_mail.status.success() {
            return Err(Error::GitError(from_utf8(&git_mail.stderr)?.to_string()));
        }

        (
            from_utf8(&git_author.stdout)?.to_string(),
            from_utf8(&git_mail.stdout)?.to_string(),
        )
    };

    let mut maintainer = format!("{} <{}>", git_details.0, git_details.1,);

    maintainer = maintainer.replace("\n", "");

    Ok(maintainer)
}

pub fn write_checksum(dwnld_url: &str) -> Result<String, Error> {
    debug!("GET: {}", dwnld_url);

    info!("Downloading distfile to generate checksum...");

    let resp = reqwest::get(dwnld_url);

    if resp.is_err() {
        return Err(Error::ShaError(resp.unwrap_err().to_string()));
    }

    let mut hasher = Sha256::new();

    let hash_res = resp.unwrap().copy_to(&mut hasher);

    if hash_res.is_err() {
        return Err(Error::ShaError(hash_res.unwrap_err().to_string()));
    }

    let hash = hasher.result();

    debug!("Hash: {:x}", &hash);

    Ok(format!("{:x}", &hash))
}