use helpers::*;
use types::*;

// Query the metacpan.org API. Returns a PkgInfo that contains all important info
pub fn perldist_info(perldist_name: &str) -> Result<PkgInfo, Error> {
    let client = metacpan_api::SyncClient::new();

    let query_result = client.perl_info(&perldist_name);

    let query_result = match query_result {
        Ok(query_result) => query_result,
        Err(_error) => client.perl_info(&client.get_dist(&perldist_name).map_err(|e| err_handler(&format!("Failed to info for the PerlDist {}: {} ", &perldist_name, &e.to_string()))).unwrap())?,
    };

    debug!("metacpan.org query result: {:?}", query_result);

    let pkg_info = PkgInfo {
        pkg_name: "perl-".to_string() + &query_result.name,
        version: query_result.version,
        description: query_result
            .description
            .unwrap_or_else(|| missing_field_s("description")),
        homepage: query_result.resources
            .homepage
            .unwrap_or_else(|| format!("https://metacpan.org/release/{}", perldist_name)),
        license: query_result.license
            .unwrap_or_else(|| missing_field_v("license")),
        dependencies: Some(order_perldeps(query_result.dependency.unwrap_or_default())),
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

    if ! make_vec.contains(&"perl".to_string()) {
        make_vec.push("perl".to_string());
    }

    if ! run_vec.contains(&"perl".to_string()) {
        run_vec.push("perl".to_string());
    }

    Dependencies {
        host: Some(vec!["perl".to_string()]),
        make: Some(make_vec),
        run: Some(run_vec)
    }
}

pub fn perldist_dep_graph(perldist_name: &str, force_overwrite: bool) {
    let client = metacpan_api::SyncClient::new();

    let query_result = client
        .perl_info(perldist_name)
        .map_err(|e| err_handler(&e.to_string()))
        .unwrap();

    let mut deps_vec = Vec::new();

    let dependencies = order_perldeps(query_result.dependency.unwrap_or_default());

    for x in dependencies.make {
        for y in x {
            deps_vec.push(y);
        }
    }

    for x in dependencies.run {
        for y in x {
            deps_vec.push(y);
        }
    }

    let xdistdir = xdist_files();

    recursive_deps(&deps_vec, &xdistdir, &PkgType::PerlDist, force_overwrite);
}