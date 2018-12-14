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
use crate::helpers::*;
use crate::types::*;
use log::debug;

/// Query the crates.io API.
///
/// # Errors
///
/// * Errors out if crates.io can't be reached
/// * Errors out if the crate can't be found on crates.io
/// * Errors if the native deps can't be determined (via `check_native_deps`)
// TODO: Switch to AsyncClient
pub(super) fn crate_info(crate_name: &str) -> Result<PkgInfo, Error> {
    let client = crates_io_api::SyncClient::new();

    let query_result = client.full_crate(crate_name, false)?;

    let crate_deps = check_native_deps(crate_name, PkgType::Crate)?;

    debug!("crates.io query result: {:?}", query_result);

    let download_url = format!(
        "https://static.crates.io/crates/{name}/{name}-${{version}}.crate",
        name = &crate_name,
    );

    let pkg_info = PkgInfo {
        pkg_name: format!("rust-{}", &crate_name),
        // gen_checksum can't replace ${version} itself, so we have to do it here
        sha: gen_checksum(&download_url.replace("${version}", &query_result.max_version))?,
        version: query_result.max_version,
        description: query_result.description,
        homepage: query_result
            .homepage
            .unwrap_or(format!("https://crates.io/crates/{}", &crate_name)),
        license: Some(vec![query_result.license.unwrap_or_default()]),
        dependencies: crate_deps,
        download_url: Some(download_url),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

// Below you can see how this function could look if we were to figure out recursive deps
// of crates. The problem is that crates.io doesn't tell us recursive deps, so we'd have
// to iterate over some 50-200 packages (and there aren't many crates with less than 100
// deps, especially for binaries), which would mean 100-400 API calls (since crates_io_api
// queries the package once and then the latest version), which is UBER SLOW!
//pub(super) fn get_crate_deps(crate_name: &str, done_deps: Option<Vec<crates_io_api::Dependency>>) -> Result<Vec<crates_io_api::Dependency>, Error> {
pub(super) fn get_crate_deps(crate_name: &str) -> Result<Vec<crates_io_api::Dependency>, Error> {
    let client = crates_io_api::SyncClient::new();

    let query_result = client.get_crate(crate_name)?;

    let latest_version = &query_result.versions[0].num;

    //let mut deps = done_deps.clone().unwrap_or(client.crate_dependencies(crate_name, &latest_version)?);

    Ok(client.crate_dependencies(crate_name, &latest_version)?)
}

/*
    for x in deps.clone() {
        if done_deps.as_ref().unwrap_or(&Vec::new()).contains(&x) {
            continue;
        }
        deps.append(&mut get_crate_deps(&x.crate_id, Some(deps.clone()))?);
    }

    Ok(deps)
}
*/
