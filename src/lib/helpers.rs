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
use crate::types::*;
use crate::template_handler;
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use std::env::var_os;
use std::path::Path;
use std::process::Command;
use std::str::from_utf8;

/// A not so pretty hack to insert an empty string if PkgInfo has a field that can't
/// be determined and an option is to cumbersome to use
pub(super) fn missing_field_s(field_name: &str) -> String {
    warn!(
        "Couldn't determine field '{}'! Please add it to the template yourself.",
        field_name
    );

    String::from("")
}

/// Figure out whether we're dealing with a crate or a gem if the user hasn't specified that.
///
/// # Errors
///
/// * Errors out of a package with the name the user gave us can be found multiple platforms
/// * Errors out if the package can't be found on any platform
pub fn figure_out_provider( pkg_name: &str) -> Result<PkgType, Error> {
    let crate_status = crates_io_api::SyncClient::new()
        .get_crate(&pkg_name)
        .is_ok();

    let gem_status = rubygems_api::SyncClient::new().gem_info(&pkg_name).is_ok();

    let perldist_status = metacpan_api::SyncClient::new().perl_info(&pkg_name).is_ok();

    if (crate_status && gem_status)
        || (crate_status && perldist_status)
        || (gem_status && perldist_status) {
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
}

/// Figure out where to write template files with `xdistdir`
///
/// # Errors
///
/// * Errors out if `xdistdir` couldn't be run
/// * Errors out if the UTF-8 `xdistdir` returns can't be converted to a String
/// * Errors out if the String returned by `xdistdir` contains a `~` and the env
///   variable `HOME` isn't set. Rust interprets `~` as `./~` instead of as `$HOME`
///   as shell does.
pub(crate) fn xdist_files() -> Result<String, Error> {
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

/// Generic function to handle recursive deps.
///
/// # Errors
/// * Errors out if `template_handler` fails to run
pub(super) fn recursive_deps(deps: &[String], xdistdir: &str, pkg_type: PkgType) -> Result<(), Error> {
    for x in deps {
        // We want to ignore built-in deps
        if !is_built_in(x, pkg_type) {
            let tmpl_path = if pkg_type == PkgType::Gem {
                format!("{}ruby-{}/template", xdistdir, x)
            } else if pkg_type == PkgType::PerlDist {
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
                template_handler(&get_pkginfo(&x, pkg_type)?, pkg_type, false, true)?;
            } else {
                debug!("Dependency {} is already satisfied!", x);
            }
        }
    }
    Ok(())
}

/// Checks the length of a string and prints a warning if it's too long for a template
pub(super) fn check_string_len(pkg_name: &str, string: &str, string_type: &str) -> String {
    if string.len() >= 80 {
        warn!(
            "{} of package {} is longer than 80 characters, please cut as you see fit!",
            pkg_name, string_type
        );
    }

    string.to_string()
}

/// Checks if a Gem or PerlDist is built into Ruby/Perl.
pub(crate) fn is_built_in(pkg_name: &str, pkg_type: PkgType) -> bool {
    if pkg_type == PkgType::Crate {
        return false;
    }

    let data: BuiltIns = serde_json::from_str(include_str!("built_in.in")).unwrap();

    let built_ins = BuiltIns {
        perl: data.perl,
        ruby: data.ruby,
    };

    if pkg_type == PkgType::Gem {
        for x in built_ins.ruby {
            if pkg_name == x.name {
                warn!(
                    "Gem {} is part of ruby, won't write a template for it!",
                    pkg_name
                );
                return true;
            }
        }
    } else if pkg_type == PkgType::PerlDist {
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

/// Generates a String that we can write to `depends` in the template
pub(super) fn gen_dep_string(dep_vec: &[String], pkg_type: PkgType) -> String {
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

        if pkg_type == PkgType::PerlDist {
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

/// Converts some non-SPDX conform names to SPDX-conform ones (e.g. GPL-2.0+ to GPL-2.0-or-later)
pub(super) fn correct_license(license: &str) -> String {
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

/// Convenience function to get PkgInfo for the package `pkg_name` of a certain PkgType
///
/// Errors if determining PkgInfo fails, see the doc for [crate_info](crate::crates::crate_info),
/// [gem_info](crate::gems::gem_info) and [perldist_info](crate::perldist::perldist_info)
pub fn get_pkginfo(pkg_name: &str, pkg_type: PkgType) -> Result<PkgInfo, Error> {
    if pkg_type == PkgType::Crate {
        crate_info(&pkg_name)
    } else if pkg_type == PkgType::PerlDist {
        perldist_info(&pkg_name)
    } else {
        gem_info(pkg_name)
    }
}

/// Gets the git author from either the environment or `git config`
///
/// # Errors
///
/// * Errors if neither `GIT_AUTHOR_NAME` _and_ `GIT_AUTHOR_EMAIL`
///   are set _and_ the git username & email can't be determined via
///   `git config`
pub(super) fn get_git_author() -> Result<String, Error> {
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

/// Download the file specified via dwnld_url and return its checksum
///
/// # Errors
///
/// * Errors out if the file can't be downloaded
/// * Errors out if the sha256sum couldn't be determined
pub(super) fn write_checksum(dwnld_url: &str) -> Result<String, Error> {
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
