use crate::helpers::*;
use crate::types::*;
use log::debug;

/// Query the metacpan.org API. If `perldist_name` is the name of a perl module, it will query
/// the perldist instead.
///
/// # Errors
/// * Errors out if metacpan.org can't be reached
/// * Errors out if the perldist (or the module it is the parent of) can't be queried.
/// * Errors out if `write_checksum` Errors.
pub fn perldist_info(perldist_name: &str) -> Result<PkgInfo, Error> {
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
        .replace(&query_result.version, "${version}");

    let pkg_info = PkgInfo {
        pkg_name: "perl-".to_string() + &query_result.name,
        version: query_result.version,
        description: query_result
            .description
            .unwrap_or_else(|| missing_field_s("description")),
        homepage: query_result
            .resources
            .homepage
            .unwrap_or_else(|| format!("https://metacpan.org/release/{}", perldist_name)),
        license: query_result
            .license
            .unwrap_or_else(|| vec![missing_field_s("license")]),
        dependencies: Some(order_perldeps(query_result.dependency.unwrap_or_default())),
        sha: write_checksum(&query_result.download_url)?,
        download_url: Some(download_url),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

fn order_perldeps(dep_vec: Vec<metacpan_api::PerlDep>) -> Dependencies {
    let mut make_vec = Vec::new();
    let mut run_vec = Vec::new();

    for x in dep_vec {
        match x.phase.as_ref() {
            "configure" => make_vec.push(x.module),
            "runtime" => run_vec.push(x.module),
            _ => (),
        }
    }

    if !make_vec.contains(&"perl".to_string()) {
        make_vec.push("perl".to_string());
    }

    if !run_vec.contains(&"perl".to_string()) {
        run_vec.push("perl".to_string());
    }

    Dependencies {
        host: Some(vec!["perl".to_string()]),
        make: Some(make_vec),
        run: Some(run_vec),
    }
}

/// Figures out recursive deps of a perldist and calls `recursive_deps` to generate templates
/// for those perldists.
///
/// # Errors
///
/// * Errors out if metacpan.org can't be reached
/// * Errors out if the perldist can't be found on metacpan.org
/// * Errors out if `xdistdir` can't be determined (via `xdist_files`)
/// * Errors out if `recursive_deps` errors
pub fn perldist_dep_graph(perldist_name: &str) -> Result<(), Error> {
    let client = metacpan_api::SyncClient::new();

    let query_result = client.perl_info(&perldist_name);

    let query_result = match query_result {
        Ok(query_result) => query_result,
        Err(_error) => client.perl_info(&client.get_dist(&perldist_name)?)?,
    };

    let mut deps_vec = Vec::new();

    let dependencies = order_perldeps(query_result.dependency.unwrap_or_default());

    for x in dependencies.make.unwrap() {
        deps_vec.push(x);
    }

    for x in dependencies.run.unwrap() {
        deps_vec.push(x);
    }

    let xdistdir = xdist_files()?;

    recursive_deps(&deps_vec, &xdistdir, &PkgType::PerlDist)?;

    Ok(())
}
