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

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    File(std::io::Error),
    #[fail(display = "{}", _0)]
    Crate(crates_io_api::Error),
    #[fail(display = "{}", _0)]
    Gem(rubygems_api::Error),
    #[fail(display = "{}", _0)]
    PerlDist(metacpan_api::Error)
}

#[derive(Debug, PartialEq)]
pub enum PkgType {
    Crate,
    Gem,
    PerlDist,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::File(e)
    }
}

impl From<crates_io_api::Error> for Error {
    fn from(e: crates_io_api::Error) -> Self {
        Error::Crate(e)
    }
}

impl From<rubygems_api::Error> for Error {
    fn from(e: rubygems_api::Error) -> Self {
        Error::Gem(e)
    }
}

impl From<metacpan_api::Error> for Error {
    fn from (e: metacpan_api::Error) -> Self {
        Error::PerlDist(e)
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
}
