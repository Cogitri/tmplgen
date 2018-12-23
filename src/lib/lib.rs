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
//! The following will write a template for `tmplgen` in `$XBPS_DISTDIR/srcpkgs/rust-tmplgen/template`
//!
//! ```
//! use libtmplgen::*;
//! use std::fs::File;
//! use std::io::prelude::*;
//!
//! fn write_template() -> Result<(), Error> {
//!     let template = TmplBuilder::new("tmplgen").get_type()?.get_info()?.generate(true)?;
//!
//!     let mut file = File::create("./template")?;
//!     file.write_all(template.inner.as_bytes())?;
//!
//!     Ok(())
//! }
//! ```
//!
//! *Wait. What?*
//! Here's a step-by-step example:
//! ```
//! use libtmplgen::*;
//! use std::fs::File;
//! use std::io::prelude::*;
//!
//! fn write_template() -> Result<(), Error> {
//!     // Creates a new TmplBuilder for the pkg "tmplgen"
//!     let mut tmpl_builder = TmplBuilder::new("tmplgen");
//!     // Get the PkgType of this crate
//!     tmpl_builder.get_type()?;
//!     // Get a PkgInfo struct of this crate
//!     tmpl_builder.get_info()?;
//!     // Generate a [Template](crate::types::Template) which we can write later on
//!     // The bool sets if we want the template to be prefixed with {perl-,ruby-,rust-}
//!     let template = tmpl_builder.generate(true)?;
//!
//!     // Create a file called "template" in the current dir
//!     let mut file = File::create("./template")?;
//!     // Write the [Template](crate::types::Template) to the file we just created
//!     file.write_all(template.inner.as_bytes())?;
//!
//!     Ok(())
//! }
//! ```
//!
//! See [TmplBuilder](crate::types::TmplBuilder) for most of the exciting other stuff.

mod crates;
mod gems;
mod helpers;
mod perldist;
#[cfg(test)]
mod tests;

pub mod errors;
pub mod tmplwriter;
pub mod types;

pub use crate::errors::*;
pub use crate::tmplwriter::*;
pub use crate::types::*;
