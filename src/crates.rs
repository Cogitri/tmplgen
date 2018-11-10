use helpers::*;
use types::*;

// Query the crates.io API. Returns a PkgInfo that contains all important info
pub fn crate_info(crate_name: &String) -> Result<PkgInfo, Error> {
    let client = crates_io_api::SyncClient::new();

    let query_result = client.full_crate(crate_name, false)?;

    debug!("crates.io query result: {:?}", query_result,);

    let pkg_info = PkgInfo {
        pkg_name: crate_name.clone(),
        version: query_result.max_version,
        description: query_result
            .description
            .unwrap_or_else(|| missing_field_s("description")),
        homepage: query_result
            .homepage
            .unwrap_or_else(|| missing_field_s("homepage")),
        license: vec![
            query_result
                .license
                .unwrap_or_else(|| missing_field_s("license")),
        ],
        dependencies: None,
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}
