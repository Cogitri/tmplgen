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

use crate::errors::Error;
use crate::types::*;
use log::debug;
use rayon::prelude::*;
use retry::retry_exponentially;

/// Query the rubygems.org API.
///
/// # Errors
///
/// * Errors out if rubygems.org can't be reached
/// * Errors out if the gem can't be found on rubygems.org
pub(super) fn gem_info(gem_name: &str) -> Result<PkgInfo, Error> {
    let client = rubygems_api::SyncClient::new();

    let query_result =
        match retry_exponentially(3, 10.0, &mut || client.gem_info(gem_name), |result| {
            result.is_ok()
        }) {
            Ok(response) => response?,
            Err(error) => return Err(Error::Gem(error.to_string())),
        };

    let mut dep_vec_run = query_result
        .dependencies
        .runtime
        .unwrap_or_default()
        .par_iter()
        .filter(|&x| {
            !TmplBuilder::new(&x.name)
                .set_type(PkgType::Gem)
                .is_built_in()
                .unwrap_or(false)
        })
        .map(|x| parse_gem_version_req(&x))
        .collect::<Vec<_>>();

    // Gems always depend on ruby.
    dep_vec_run.push("ruby".to_string());

    debug!("Gem run dependencies: {:?}", &dep_vec_run);

    let gem_run_deps = if dep_vec_run.is_empty() {
        None
    } else {
        Some(dep_vec_run)
    };

    let pkg_info = PkgInfo {
        pkg_name: format!("ruby-{}", gem_name.to_string()),
        version: query_result.version,
        description: query_result.info,
        homepage: query_result
            .homepage_uri
            .unwrap_or_else(|| format!("https://rubygems.org/gems/{}", gem_name)),
        license: query_result.licenses,
        dependencies: Some(Dependencies {
            host: None,
            make: None,
            run: gem_run_deps,
        }),
        sha: query_result.sha,
        download_url: None,
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

/* Can't be used right now, we'll just replace it with >=
// Convert the ~> comparator to something useful for us.
// The ~> comparator is meant to allow only version updates up to the first version specifier
// ~> 2.0.3 means >= 2.0.3 ∩ < 2.1
// ~> 2.1 means >= 2.1 ∩ > 3.0
// ~> 2 means >= 2 ∩ > 3
pub fn tilde_parse(version: String) -> Option<Vec<String>> {
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
        4 => Some(vec![
            String::from(">=".to_owned() + &version),
            String::from(
                "<".to_owned()
                    + &ver_vec[0]
                    + &".".to_owned()
                    + &(ver_vec[1].parse::<u8>().unwrap() + 1).to_string()
                    + &ver_vec[3].to_string(),
            ),
        ]),
        _ => None,
    }
}
*/

/// Determines the run dependencies of a gem. Deals with version requirements.
pub(super) fn parse_gem_version_req(rubygem_dep: &rubygems_api::GemRunDeps) -> String {
    let cmpr = rubygem_dep
        .requirements
        .split_whitespace()
        .collect::<Vec<_>>()[0];

    let ver = rubygem_dep
        .requirements
        .split_whitespace()
        .collect::<Vec<_>>()[1]
        .replace(",", "");

    match cmpr {
        ">" | "<" | "<=" => format!("ruby-{}{}{}", &rubygem_dep.name, cmpr, &ver),
        ">=" => {
            if ver == "0" {
                format!("ruby-{}", &rubygem_dep.name)
            } else {
                format!("ruby-{}{}{}", &rubygem_dep.name, cmpr, &ver)
            }
        }
        "~>" => format!("ruby-{}>={}", &rubygem_dep.name, &ver),
        _ => format!("ruby-{}", &rubygem_dep.name),
    }
}
