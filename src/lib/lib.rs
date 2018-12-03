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
//! use std::fs::File;
//! use std::io::prelude::*;
//!
//! let template = TmplBuilder::new("tmplgen").get_type().unwrap().get_info().unwrap().generate().unwrap();
//!
//! let mut file = File::create("./template").unwrap();
//! file.write_all(template.inner.as_bytes()).unwrap();
//! ```
//!
//! *Wait. What?*
//! Here's a step-by-step example:
//! ```
//! use libtmplgen::*;
//! use std::fs::File;
//! use std::io::prelude::*;
//!
//! // Creates a new TmplBuilder for the pkg "tmplgen"
//! let mut tmpl_builder = TmplBuilder::new("tmplgen");
//! // Get the PkgType of this crate
//! tmpl_builder.get_type().unwrap();
//! // Get a PkgInfo struct of this crate
//! tmpl_builder.get_info().unwrap();
//! // Generate a [Template](crate::types::Template) which we can write later on
//! let template = tmpl_builder.generate().unwrap();
//!
//! // Create a file called "template" in the current dir
//! let mut file = File::create("./template").unwrap();
//! // Write the [Template](crate::types::Template) to the file we just created
//! file.write_all(template.inner.as_bytes()).unwrap();
//! ```
//!
//! See [TmplBuilder](crate::types::TmplBuilder) for most of the exciting other stuff.

mod crates;
mod gems;
mod perldist;
mod helpers;
#[cfg(test)]
mod tests;

pub mod tmplwriter;
pub mod types;
pub mod errors;

pub use crate::types::*;
pub use crate::tmplwriter::*;
pub use crate::errors::*;