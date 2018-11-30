use helpers::*;
use types::*;

// Query the metacpan.org API. Returns a PkgInfo that contains all important info
pub fn perldist_info(perldist_name: &str) -> Result<PkgInfo, Error> {
    let client = metacpan_api::SyncClient::new();

    let query_result = client.perl_info(&perldist_name);

    let query_result = match query_result {
        Ok(query_result) => query_result,
        Err(_e) => client.perl_info(
            &client
                .get_dist(&perldist_name)
                .map_err(|e| return Error::PerlDist(e.to_string()))
                .unwrap(),
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
        sha: None,
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
