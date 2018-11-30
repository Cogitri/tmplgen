use crate::helpers::*;
use crate::types::*;
use log::debug;

// Query the crates.io API. Returns a PkgInfo that contains all important info
pub fn crate_info(crate_name: &str) -> Result<PkgInfo, Error> {
    let client = crates_io_api::SyncClient::new();

    let query_result = client.full_crate(crate_name, false)?;

    let crate_deps = check_native_deps(crate_name)?;

    debug!("crates.io query result: {:?}", query_result,);

    let download_url = format!(
        "https://static.crates.io/crates/{name}/{name}-${{version}}.crate",
        name = &crate_name,
    );

    // We don't want to have $-{version} in our download_url
    let sha_download_url = format!(
        "https://static.crates.io/crates/{name}/{name}-{version}.crate",
        name = &crate_name,
        version = query_result.max_version,
    );

    let pkg_info = PkgInfo {
        pkg_name: crate_name.to_string(),
        version: query_result.max_version,
        description: query_result
            .description
            .unwrap_or_else(|| missing_field_s("description")),
        homepage: query_result
            .homepage
            .unwrap_or_else(|| missing_field_s("homepage")),
        license: vec![query_result
            .license
            .unwrap_or_else(|| missing_field_s("license"))],
        dependencies: crate_deps,
        sha: write_checksum(&sha_download_url)?,
        download_url: Some(download_url),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

fn get_crate_deps(crate_name: &str) -> Result<Vec<crates_io_api::Dependency>, Error> {
    let client = crates_io_api::SyncClient::new();

    let query_result = client.get_crate(crate_name)?;

    let latest_version = &query_result.versions[0].num;

    Ok(client.crate_dependencies(crate_name, &latest_version)?)
}

// Check if a crate needs native libs (e.g. libressl-devel)
// TODO: This only works with direct deps!
pub fn check_native_deps(crate_name: &str) -> Result<Option<Dependencies>, Error> {
    let dependencies = get_crate_deps(crate_name)?;

    debug!("Crate dependencies: {:?}", dependencies);

    let mut make_dep_vec = vec![];

    for x in dependencies {
        if x.crate_id == "openssl-sys" {
            make_dep_vec.push("libressl-devel".to_string());
        }
    }

    if !make_dep_vec.is_empty() {
        Ok(Some(Dependencies {
            host: Some(vec!["pkg-config".to_string()]),
            make: Some(make_dep_vec),
            run: None,
        }))
    } else {
        Ok(None)
    }
}
