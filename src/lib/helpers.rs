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
use crate::errors::Error;
use crate::gems::*;
use crate::perldist::*;
use crate::types::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use std::env::var_os;
use std::process::Command;
use std::str::from_utf8;

/// Figure out whether we're dealing with a crate or a gem if the user hasn't specified that.
///
/// # Errors
///
/// * Errors out of a package with the name the user gave us can be found multiple platforms
/// * Errors out if the package can't be found on any platform
pub(super) fn figure_out_provider(pkg_name: &str) -> Result<PkgType, Error> {
    //TODO: Actually check that the error is "Not Found"!
    let crate_status = crates_io_api::SyncClient::new()
        .get_crate(&pkg_name)
        .is_ok();

    let gem_status = rubygems_api::SyncClient::new().gem_info(&pkg_name).is_ok();

    let perldist_status = metacpan_api::SyncClient::new().perl_info(&pkg_name).is_ok();

    if (crate_status && gem_status)
        || (crate_status && perldist_status)
        || (gem_status && perldist_status)
    {
        let mut found_platforms = Vec::new();
        if crate_status {
            found_platforms.push("crates.io");
        }
        if gem_status {
            found_platforms.push("metacpan.org");
        }
        if perldist_status {
            found_platforms.push("rubygems.org")
        }

        Err(Error::AmbPkg(
            (&format!("{} on the platforms {:?}", pkg_name, found_platforms)
                .replace("[", "")
                .replace("]", ""))
                .to_string(),
        ))
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

/// Generates a String that we can write to `depends` in the template
pub(super) fn gen_dep_string(dep_vec: &[String], pkg_type: PkgType) -> String {
    let mut dep_string = String::new();

    for x in dep_vec {
        let after_string = format!("{}{}", &dep_string, x);

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
                dep_string.push_str(x)
            } else {
                dep_string.push_str(&format!("perl-{}", &x.replace("::", "-")));
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
pub(super) fn get_pkginfo(pkg_name: &str, pkg_type: PkgType) -> Result<PkgInfo, Error> {
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
        match Command::new("git").args(&["--version"]).output() {
            Ok(_) => {}
            Err(e) => {
                if let std::io::ErrorKind::NotFound = e.kind() {
                    return Err(Error::GitError("Couldn't find the command `git`. Make sure you have installed git and that it's in your PATH, or set GIT_AUTHOR_NAME and GIT_AUTHOR_EMAIL!".to_string()));
                }
            }
        }

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

    let maintainer = format!("{} <{}>", git_details.0, git_details.1).replace("\n", "");

    Ok(maintainer)
}

/// Download the file specified via dwnld_url and return its checksum
///
/// # Errors
///
/// * Errors out if the file can't be downloaded
/// * Errors out if the sha256sum couldn't be determined
pub(super) fn gen_checksum(dwnld_url: &str) -> Result<String, Error> {
    let req_client = reqwest::Client::new();
    let url = reqwest::Url::parse(dwnld_url)?;

    let total_size = {
        let resp = req_client.head(url.as_str()).send()?;
        if resp.status().is_success() {
            resp.headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse().ok())
                .unwrap_or(0)
        } else {
            return Err(Error::ShaError(format!(
                "Couldn't download URL: {}. Error: {:?}",
                url,
                resp.status(),
            )));
        }
    };

    debug!("GET: {}", dwnld_url);

    info!("Downloading distfile to generate checksum...");

    let resp = req_client.get(url.as_str());

    // Do not display a progresssbar if the download is under 200KiB big,
    // it usually is either not visible or just flashes over the screen anyway.
    let pb = if total_size > 200_000 {
        ProgressBar::new(total_size)
    } else {
        ProgressBar::hidden()
    };

    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));

    let mut source = DownloadProgress {
        progress_bar: pb,
        inner: resp.send()?,
    };

    let mut hasher = Sha256::new();

    let hash_res = std::io::copy(&mut source, &mut hasher);

    if hash_res.is_err() {
        return Err(Error::ShaError(hash_res.unwrap_err().to_string()));
    }

    let hash = hasher.result();

    debug!("Hash: {:x}", &hash);

    Ok(format!("{:x}", &hash))
}
