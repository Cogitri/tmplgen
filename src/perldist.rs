use helpers::*;
use types::*;

// Query the metacpan.org API. Returns a PkgInfo that contains all important info
pub fn perldist_info(perldist_name: &str) -> Result<PkgInfo, Error> {
    let client = metacpan_api::SyncClient::new();

    let query_result = client.perl_info(&perldist_name)?;

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
        license: query_result.license,
        dependencies: Some(Dependencies {
            host: Some(vec!["perl".to_string()]),
            make: Some(vec!["perl".to_string()]),
            run: Some(vec!["$makedepends".to_string()])
        }),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    warn!("Packages of the type `perldist` don't support automatic dependency detection yet.");

    Ok(pkg_info)
}