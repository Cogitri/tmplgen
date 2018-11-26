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

use helpers::*;
use types::*;

// Returns a PkgInfo object that contains all the info relevant for us
pub fn gem_info(gem_name: &str) -> Result<PkgInfo, Error> {
    let client = rubygems_api::SyncClient::new();

    let query_result = client.gem_info(gem_name)?;

    let mut dep_vec_run = Vec::new();

    for x in query_result.dependencies.runtime.unwrap() {
        let dep = determine_gem_run_deps(&x)?;
        dep_vec_run.push(dep);
    }

    debug!("Gem run dependencies: {:?}", dep_vec_run);

    let pkg_info = PkgInfo {
        pkg_name: format!("ruby-{}", gem_name.to_string()),
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
            host: None,
            make: None,
            run: Some(dep_vec_run),
        }),
    };

    debug!("All pkg related info: {:?}", pkg_info);

    Ok(pkg_info)
}

// If the gem has recursive deps, we should also generate templates for those if they
// don't exist already
pub fn gem_dep_graph(gem_name: &str, force_overwrite: bool) {
    let client = rubygems_api::SyncClient::new();

    let query_result = client
        .gem_info(gem_name)
        .map_err(|e| err_handler(&format!("Failed to query gem {}: {}", &gem_name,  e.to_string())))
        .unwrap();

    let mut deps_vec = Vec::new();

    for x in query_result.dependencies.runtime.unwrap() {
        deps_vec.push(x.name);
    }

    let xdistdir = xdist_files();

    recursive_deps(&deps_vec, &xdistdir, &PkgType::Gem, force_overwrite);
}

/* Can't be used right now we'll just replace it with >=
// Convert the ~> comparator to something useful for us.
// The ~> comparator is meant to allow only version updates up to the first version specifier
// ~> 2.0.3 means >= 2.0.3 ∩ < 2.1
// ~> 2.1 means >= 2.1 ∩ > 3.0
// ~> 2 means >= 2 ∩ > 3
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
*/

// Determine the run dependencies of a gem. Deals with version requirements.
fn determine_gem_run_deps(rubygem_dep: &rubygems_api::GemRunDeps) -> Result<String, Error> {
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
        "~>" => "ruby-".to_string() + &rubygem_dep.name + ">=" + &ver,
        _ => "ruby-".to_string() + &rubygem_dep.name,
    };

    Ok(ver_req)
}
