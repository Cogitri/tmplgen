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

use serde_derive::Deserialize;
use std::io::Read;

/// The TemplateBuilder struct, which is used to build a [Template](crate::types::Template)
pub struct TmplBuilder {
    pub pkg_name: String,
    pub pkg_type: Option<PkgType>,
    pub pkg_info: Option<PkgInfo>,
    pub deps: Option<Vec<String>>,
}

pub struct Template {
    pub inner: String,
    pub name: String,
}

impl Read for Template {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.as_bytes().read(buf)
    }
}

pub(super) struct DownloadProgress<R> {
    pub inner: R,
    pub progress_bar: indicatif::ProgressBar,
}

impl<R: std::io::Read> std::io::Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

/// The PkgType enum, containing all types of packages tmplgen can handle
#[derive(Copy, Clone, Eq, Ord, PartialOrd, Hash, Debug, PartialEq)]
pub enum PkgType {
    Crate,
    Gem,
    PerlDist,
}

/// The Dependencies struct that contains all dependencies a package might have
#[derive(Clone, Eq, Ord, PartialOrd, Hash, Default, Debug, PartialEq)]
pub struct Dependencies {
    pub host: Option<Vec<String>>,
    pub make: Option<Vec<String>>,
    pub run: Option<Vec<String>>,
}

/// The PkgInfo struct, that contains all info relevant to the package
#[derive(Clone, Eq, Ord, PartialOrd, Hash, Default, Debug, PartialEq)]
pub struct PkgInfo {
    pub pkg_name: String,
    pub version: String,
    pub description: Option<String>,
    pub homepage: String,
    pub license: Option<Vec<String>>,
    pub dependencies: Option<Dependencies>,
    pub sha: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BuiltIns {
    pub perl: Vec<BuiltInDep>,
    pub ruby: Vec<BuiltInDep>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BuiltInDep {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CorrectedVals {
    pub licenses: Vec<CorrectedLicenses>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CorrectedLicenses {
    pub is: String,
    pub should: String,
}
