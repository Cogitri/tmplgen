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
        version: query_result.max_version.clone(),
        description: query_result.description,
        homepage: query_result
            .homepage
            .unwrap_or(format!("https://crates.io/crates/{}", &crate_name)),
        license: Some(vec![query_result.license.unwrap_or_default()]),
        dependencies: crate_deps,
        // gen_checksum can't replace ${version} itself, so we have to do it here
        sha: gen_checksum(&download_url.replace("${version}", &query_result.max_version))?,
        download_url: Some(download_url),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

pub(super) fn get_crate_deps(crate_name: &str) -> Result<Vec<crates_io_api::Dependency>, Error> {
    let client = crates_io_api::SyncClient::new();

    let query_result = client.get_crate(crate_name)?;

    let latest_version = &query_result.versions[0].num;

    Ok(client.crate_dependencies(crate_name, &latest_version)?)
}