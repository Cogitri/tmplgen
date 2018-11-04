use crates::*;
use gems::*;

#[test]
fn test_crate() {
    let pkg_info = crate_info(&"rubygems_api".to_string()).unwrap();
    assert!(pkg_info.homepage == "https://github.com/Cogitri/rubygems_api");
}

#[test]
fn test_gem() {
    let pkg_info = gem_info(&"ffi".to_string()).unwrap();
    assert!(pkg_info.license[0] == "BSD-3-Clause");
}