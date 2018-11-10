#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "{}", _0)]
    File(std::io::Error),
    #[fail(display = "{}", _0)]
    Crate(crates_io_api::Error),
    #[fail(display = "{}", _0)]
    Gem(rubygems_api::Error),
}

#[derive(Debug, PartialEq)]
pub enum PkgType {
    Crate,
    Gem,
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

#[derive(Debug)]
pub struct Dependencies {
    pub make: Vec<String>,
    pub run: Vec<String>,
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
