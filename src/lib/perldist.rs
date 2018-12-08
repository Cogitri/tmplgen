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

/// Query the metacpan.org API. If `perldist_name` is the name of a perl module, it will query
/// the perldist instead.
///
/// # Errors
/// * Errors out if metacpan.org can't be reached
/// * Errors out if the perldist (or the module it is the parent of) can't be queried.
/// * Errors out if `gen_checksum` Errors.
pub(super) fn perldist_info(perldist_name: &str) -> Result<PkgInfo, Error> {
    let client = metacpan_api::SyncClient::new();

    let query_result = client.perl_info(&perldist_name);

    let query_result = match query_result {
        Ok(query_result) => query_result,
        Err(_e) => client.perl_info(
            &client
                .get_dist(&perldist_name)
                .map_err(|e| Error::PerlDist(e.to_string()))?,
        )?,
    };

    debug!("metacpan.org query result: {:?}", query_result);

    let download_url = query_result
        .download_url
        .replace(&query_result.version.as_str().unwrap_or_default(), "${version}");

    let pkg_info = PkgInfo {
        pkg_name: "perl-".to_string() + &query_result.name,
        version: query_result.version.as_str().unwrap_or_default().to_string(),
        description: query_result.description.clone(),
        homepage: query_result.resources.homepage.clone().unwrap_or_else(|| format!("https://metacpan.org/pod/{}", query_result.name)),
        license: Some(query_result.license.unwrap_or_default()),
        dependencies: Some(order_perldeps(query_result.dependency.unwrap_or_default())?),
        sha: gen_checksum(&query_result.download_url)?,
        download_url: Some(download_url),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

fn order_perldeps(dep_vec: Vec<metacpan_api::PerlDep>) -> Result<Dependencies, Error> {
    let mut make_vec = Vec::new();
    let mut run_vec = Vec::new();

    let client = metacpan_api::SyncClient::new();

    for x in dep_vec {
        if TmplBuilder::new(&x.module).set_type(PkgType::PerlDist).is_built_in().unwrap_or({ false }) {
            continue;
        }

        let query_result = client.perl_info(&x.module);

        let query_result = match query_result {
            Ok(query_result) => query_result,
            Err(_e) => client.perl_info(
                &client
                    .get_dist(&x.module)
                    .map_err(|e| Error::PerlDist(e.to_string()))?,
            )?,
        };

        match x.phase.as_ref() {
            "configure" => {
                if ! make_vec.contains(&query_result.name) {
                    make_vec.push(query_result.name)
                }
            },
            "runtime" => {
                if ! run_vec.contains(&query_result.name) {
                    run_vec.push(query_result.name)
                } },
            _ => (),
        }
    }

    if !make_vec.contains(&"perl".to_string()) {
        make_vec.push("perl".to_string());
    }

    if !run_vec.contains(&"perl".to_string()) {
        run_vec.push("perl".to_string());
    }

    Ok(Dependencies {
        host: Some(vec!["perl".to_string()]),
        make: Some(make_vec),
        run: Some(run_vec),
    })
}

/// Figures out recursive deps of a perldist and calls `recursive_deps` to generate templates
/// for those perldists.
///
/// # Errors
///
/// * Errors out if metacpan.org can't be reached
/// * Errors out if the perldist can't be found on metacpan.org
/// * Errors out if `recursive_deps` errors
pub(super) fn perldist_dep_graph(perldist_name: &str) -> Result<Vec<String>, Error> {
    let client = metacpan_api::SyncClient::new();

    let query_result = client.perl_info(&perldist_name);

    let query_result = match query_result {
        Ok(query_result) => query_result,
        Err(_error) => client.perl_info(&client.get_dist(&perldist_name)?)?,
    };

    let mut deps_vec = Vec::new();

    let dependencies = order_perldeps(query_result.dependency.unwrap_or_default())?;

    for x in dependencies.make.unwrap() {
        deps_vec.push(x);
    }

    for x in dependencies.run.unwrap() {
        deps_vec.push(x);
    }

    Ok(deps_vec)
}
