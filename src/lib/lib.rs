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

//! libtmplgen can be used for querying different language-specific package managers and generating
//! Void Linux build templates for them. Currently the following providers are supported:
//!
//! * [crates.io](https://crates.io)
//! * [metacpan.org](https://metacpan.org)
//! * [rubygems.org](https://rubygems.org)
//!
//! # Usage
//!
//! The following will write a template for `tmplgen` in $XBPS_DISTDIR/srcpkgs/tmplgen/template
//!
//! ```
//! use libtmplgen::*;
//!
//! // Get the PkgType of this crate
//! let pkg_type = figure_out_provider("tmplgen").unwrap();
//! // Get a PkgInfo struct of this crate
//! let pkg_info = get_pkginfo("tmplgen", &pkg_type).unwrap();
//! // Don't overwrite existing templates
//! let force_overwrite = false;
//! // This isn't a recursive dep, error out if there's an error
//! let is_rec = false;
//!
//! template_handler(&pkg_info, &pkg_type, force_overwrite, is_rec);
//! ```

mod crates;
mod gems;
mod perldist;

pub mod helpers;
pub mod tmplwriter;
pub mod types;
#[cfg(test)]
mod tests;

use log::{info, warn};

use crate::perldist::*;
use crate::gems::*;

pub use crate::types::*;
pub use crate::tmplwriter::*;
pub use crate::helpers::*;

/// Handle getting the necessary info and writing a template for it. Invoked every time a template
/// should be written, especially useful for recursive deps.
///
/// # Example
///
/// ```
/// use libtmplgen::*;
///
/// // Get the PkgType of this crate
/// let pkg_type = figure_out_provider("tmplgen").unwrap();
/// // Get a PkgInfo struct of this crate
/// let pkg_info = get_pkginfo("tmplgen", &pkg_type).unwrap();
/// // Don't overwrite existing templates
/// let force_overwrite = false;
/// // This isn't a recursive dep, error out if there's an error
/// let is_rec = false;
///
/// template_handler(&pkg_info, &pkg_type, force_overwrite, is_rec);
/// ```
///
/// # Errors
///
/// * Errors out if `write_template` throws an Error, unless `is_rec` is true - this shouldn't
///   Error if a template for a recursive dep couldn't be written.
pub fn template_handler(
    pkg_info: &PkgInfo,
    pkg_type: &PkgType,
    force_overwrite: bool,
    is_rec: bool,
) -> Result<(), Error> {
    let pkg_name = &pkg_info.pkg_name;

    if is_built_in(pkg_name, pkg_type) {
        return Err(Error::BuiltIn(pkg_name.to_string()));
    }

    info!(
        "Generating template for package {} of type {:?}",
        &pkg_name, pkg_type
    );

    if is_rec {
        write_template(&pkg_info, force_overwrite, &pkg_type)
            .map_err(|e| warn!("Failed to write the template for dep {}: {}", pkg_name, e))
            .unwrap_or_default()
    } else {
        write_template(&pkg_info, force_overwrite, &pkg_type)?;
    }

    if pkg_type == &PkgType::Crate {
        return Ok(());
    }

    let dep_graph = if pkg_type == &PkgType::Gem {
        gem_dep_graph(&pkg_name.replace("ruby-", ""))
    } else {
        perldist_dep_graph(&pkg_name.replace("perl-", ""))
    };

    if dep_graph.is_err() {
        warn!(
            "Failed to write templates for all recursive deps of {}! Error: {}",
            pkg_name,
            dep_graph.unwrap_err()
        );
    }

    Ok(())
}