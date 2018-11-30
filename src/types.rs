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

use failure::Fail;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Failed to read/write the template! Error: {}", _0 )]
    File(String),
    #[fail(display = "Failed to query the crate! Error: {}", _0)]
    Crate(String),
    #[fail(display = "Failed to query the gem! Error: {}", _0)]
    Gem(String),
    #[fail(display = "Failed to query the perldist! Error: {}", _0)]
    PerlDist(String),
    #[fail(display = "Failed to convert UTF-8 to a string! Error: {}", _0)]
    UTF8(String),
    #[fail(display = "Error: {}", _0)]
    Failure(String),
    #[fail(display = "Failed to write the template! Error: {}", _0 )]
    TmplWriter(String),
    #[fail(display = "Failed to update the template! Error: {}", _0)]
    TmplUpdater(String),
    #[fail(display = "Failed to determine git username/email from environment or git config! Error: {}", _0)]
    GitError(String),
    #[fail(display = "Failed to determine XBPS_XDISTDIR: {}", _0)]
    XdistError(String),
    #[fail(display = "Found a package matching the specified package {} on multiple platforms! Please explicitly choose one via the `-t` parameter!", _0)]
    AmbPkg(String),
    #[fail(display = "Unable to determine what type of the target package {} is! Make sure you've spelled the package name correctly!", _0)]
    NoSuchPkg(String),
    #[fail(display = "Failed to write checksum to the newly written template! Error: {}", _0)]
    ShaError(String),
}

#[derive(Debug, PartialEq)]
pub enum PkgType {
    Crate,
    Gem,
    PerlDist,
}

impl From<crates_io_api::Error> for Error {
    fn from(e: crates_io_api::Error) -> Self {
        Error::Crate(e.to_string())
    }
}

impl From<rubygems_api::Error> for Error {
    fn from(e: rubygems_api::Error) -> Self {
        Error::Gem(e.to_string())
    }
}

impl From<metacpan_api::Error> for Error {
    fn from(e: metacpan_api::Error) -> Self {
        Error::PerlDist(e.to_string())
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::UTF8(e.to_string())
    }
}

impl From<failure::Error> for Error {
    fn from(e: failure::Error) -> Self {
        Error::Failure(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from (e: std::io::Error) -> Self {
        Error::File(e.to_string())
    }
}

#[derive(Debug)]
pub struct Dependencies {
    pub host: Option<Vec<String>>,
    pub make: Option<Vec<String>>,
    pub run: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct PkgInfo {
    pub pkg_name: String,
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub license: Vec<String>,
    pub dependencies: Option<Dependencies>,
    pub sha: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BuiltIns {
    pub perl: Vec<BuiltInDep>,
    pub ruby: Vec<BuiltInDep>,
}

#[derive(Debug, Deserialize)]
pub struct BuiltInDep {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CorrectedVals {
    pub licenses: Vec<CorrectedLicenses>,
}

#[derive(Debug, Deserialize)]
pub struct CorrectedLicenses {
    pub is: String,
    pub should: String,
}
