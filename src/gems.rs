use helpers::*;
use types::*;

pub fn gem_info(gem_name: &String) -> Result<PkgInfo, Error> {
    let client = rubygems_api::SyncClient::new();

    let query_result = client.gem_info(gem_name)?;

    let mut dep_vec_dev = Vec::new();

    for x in query_result.dependencies.development.unwrap() {
        dep_vec_dev.push("ruby-".to_string() + &x.name + " ");
    }

    debug!("Gem make dependencies: {:?}", dep_vec_dev);

    let mut dep_vec_run = Vec::new();

    for x in query_result.dependencies.runtime.unwrap() {
        let dep = determine_gem_run_deps(&x)?;
        dep_vec_run.push(dep);
    }

    debug!("Gem run dependencies: {:?}", dep_vec_run);

    let pkg_info = PkgInfo {
        pkg_name: format!("ruby-{}", gem_name.clone()),
        version: query_result.version,
        description: query_result
            .info
            .unwrap_or_else(|| missing_field_s("description")),
        homepage: query_result
            .homepage_uri
            .unwrap_or_else(|| missing_field_s("homepage")),
        license: query_result
            .licenses
            .unwrap_or_else(|| missing_field_v("license")),
        dependencies: Some(Dependencies {
            make: dep_vec_dev,
            run: dep_vec_run,
        }),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

pub fn gem_dep_graph(gem_name: &String, force_overwrite: bool) {
    let client = rubygems_api::SyncClient::new();

    let query_result = client.gem_info(gem_name).unwrap();

    let mut deps_vec = Vec::new();
    for x in query_result.dependencies.development.unwrap() {
        deps_vec.push(x.name);
    };

    for x in query_result.dependencies.runtime.unwrap() {
        deps_vec.push(x.name);
    };

    let xdistdir = xdist_files();

    recursive_deps(&deps_vec, &xdistdir, PkgType::Gem, force_overwrite);
    recursive_deps(&deps_vec, &xdistdir, PkgType::Gem, force_overwrite);
}

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

fn determine_gem_run_deps(
    rubygem_dep: &rubygems_api::GemRunDeps,
) -> Result<String, Error> {
    let cmpr = String::from(
        rubygem_dep
            .requirements
            .split_whitespace()
            .collect::<Vec<_>>()[0],
    );

    let ver = String::from(
        rubygem_dep
            .requirements
            .split_whitespace()
            .collect::<Vec<_>>()[1],
    );

    let ver_req = match cmpr.as_ref() {
        ">" | "<" | "<=" => "ruby-".to_string() + &rubygem_dep.name + &cmpr + &ver,
        ">=" => if ver == "0" {
            "ruby-".to_string() + &rubygem_dep.name
        } else {
            "ruby-".to_string() + &rubygem_dep.name + &cmpr + &ver
        },
        "~>" => {
            let tilde_vec = tilde_parse(ver).unwrap();
            "ruby-".to_string()
                + &rubygem_dep.name
                + &tilde_vec[0]
                + &" ruby-".to_string()
                + &rubygem_dep.name
                + &tilde_vec[1]
                + &" ".to_string()
        }
        _ => "ruby-".to_string() + &rubygem_dep.name
    };

    Ok(ver_req)
}