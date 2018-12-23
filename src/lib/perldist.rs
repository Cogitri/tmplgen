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
use rayon::prelude::*;
use retry::retry_exponentially;

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

    // Determine the query result. If the Error is "Not Found" the user either tries to
    // query an non-existent perldist _or_ it's a module. In that case we want to try to
    // query the module to get the perldist it belongs to and query the perldist module times.
    let query_result = match query_result {
        Ok(query_result) => query_result,
        Err(query_err) => {
            // TODO: Properly match this!
            if query_err.to_string() == "Not found" {
                match retry_exponentially(
                    3,
                    10.0,
                    &mut || client.perl_info(&client.get_dist(&perldist_name)?),
                    |result| result.is_ok(),
                ) {
                    Ok(response) => response?,
                    Err(error) => return Err(Error::Gem(error.to_string())),
                }
            } else {
                // If the Error isn't "Not Found" we're most likely dealing with a network error,
                // so let's retry this a few times
                match retry_exponentially(
                    3,
                    10.0,
                    &mut || client.perl_info(&perldist_name),
                    |result| result.is_ok(),
                ) {
                    Ok(response) => response?,
                    Err(error) => return Err(Error::PerlDist(error.to_string())),
                }
            }
        }
    };

    debug!("metacpan.org query result: {:?}", query_result);

    let download_url = query_result.download_url.replace(
        &query_result.version.as_str().unwrap_or_default(),
        "${version}",
    );

    let pkg_info = PkgInfo {
        pkg_name: format!("perl-{}", &query_result.name),
        version: query_result
            .version
            .as_str()
            .unwrap_or_default()
            .to_string(),
        homepage: query_result
            .resources
            .homepage
            .clone()
            .unwrap_or_else(|| format!("https://metacpan.org/pod/{}", query_result.name)),
        description: query_result.description,
        license: Some(query_result.license.unwrap_or_default()),
        dependencies: Some(order_perldeps(
            &query_result.dependency.unwrap_or_default(),
        )?),
        sha: gen_checksum(&query_result.download_url)?,
        download_url: Some(download_url),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

fn order_perldeps(dep_vec: &Vec<metacpan_api::PerlDep>) -> Result<Dependencies, Error> {
    let client = metacpan_api::SyncClient::new();

    let make_vec: Result<Vec<String>, Error> = dep_vec
        .par_iter()
        .filter(|&x| {
            !TmplBuilder::new(&x.module)
                .set_type(PkgType::PerlDist)
                .is_built_in()
                .unwrap_or(false)
        })
        .filter(|&x| x.phase == "configure")
        .map(|x| {
            let query_result = client.perl_info(&x.module);

            let result = match query_result {
                Ok(query_result) => query_result,
                Err(_) => client.perl_info(
                    &client
                        .get_dist(&x.module)
                        .map_err(|e| Error::PerlDist(e.to_string()))?,
                )?,
            };

            Ok(result.name)
        })
        .collect();

    let run_vec: Result<Vec<String>, Error> = dep_vec
        .par_iter()
        .filter(|&x| {
            !TmplBuilder::new(&x.module)
                .set_type(PkgType::PerlDist)
                .is_built_in()
                .unwrap_or(false)
        })
        .filter(|&x| x.phase == "runtime")
        .map(|x| {
            let query_result = client.perl_info(&x.module);

            let result = match query_result {
                Ok(query_result) => query_result,
                Err(_) => client.perl_info(
                    &client
                        .get_dist(&x.module)
                        .map_err(|e| Error::PerlDist(e.to_string()))?,
                )?,
            };

            Ok(result.name)
        })
        .collect();

    Ok(Dependencies {
        host: Some(vec!["perl".to_string()]),
        make: Some(make_vec?),
        run: Some(run_vec?),
    })
}
