use failure::Fail;

/// The Error enum containing all Errors that may occur when running tmplgen
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Fail)]
pub enum Error {
    #[fail(display = "Failed to read/write the template! Error: {}", _0)]
    File(String),
    #[fail(display = "Failed to query the crate! Error: {}", _0)]
    Crate(String),
    #[fail(display = "Failed to query the gem! Error: {}", _0)]
    Gem(String),
    #[fail(display = "Failed to query the perldist! Error: {}", _0)]
    PerlDist(String),
    #[fail(display = "Failed to convert UTF-8 to a string! Error: {}", _0)]
    UTF8(String),
    #[fail(display = "Failed to write the template! Error: {}", _0)]
    TmplWriter(String),
    #[fail(display = "Failed to update the template! Error: {}", _0)]
    TmplUpdater(String),
    #[fail(display = "Won't write package for built-in template {}", _0)]
    BuiltIn(String),
    #[fail(
        display = "Failed to determine git username/email from environment or git config! Error: {}",
        _0
    )]
    Git(String),
    #[fail(display = "Failed to determine XBPS_DISTDIR: {}", _0)]
    Xdist(String),
    #[fail(
        display = "Found a package matching the specified package {}! Please explicitly choose one via the `-t` parameter!",
        _0
    )]
    AmbPkg(String),
    #[fail(
        display = "Unable to determine what type of the target package {} is! Make sure you've spelled the package name correctly!",
        _0
    )]
    NoSuchPkg(String),
    #[fail(
        display = "Failed to write checksum to the newly written template! Error: {}",
        _0
    )]
    Sha(String),
    #[fail(display = "Didn't provide enough info for action {}", _0)]
    TooLittleInfo(String),
    #[fail(
        display = "Failed to write templates for all recursive deps of {}! Error: {}",
        pkg_name, err
    )]
    RecDeps { pkg_name: String, err: String },
    #[fail(display = "Can't run method {}! {}", method, err)]
    WrongUsage { method: String, err: String },
    #[fail(display = "{}", _0)]
    Reqwest(String),
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

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::File(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e.to_string())
    }
}

impl From<reqwest::UrlError> for Error {
    fn from(e: reqwest::UrlError) -> Self {
        Error::Reqwest(e.to_string())
    }
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::Git(e.to_string())
    }
}
