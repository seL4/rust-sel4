//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env::{self, VarError};
use std::path::{Path, PathBuf};

pub const SEL4_PREFIX_ENV: &str = "SEL4_PREFIX";

pub const SEL4_INCLUDE_DIRS_ENV: &str = "SEL4_INCLUDE_DIRS";

fn get_asserting_valid_unicode(var: &str) -> Option<String> {
    env::var(var)
        .map_err(|err| {
            if let VarError::NotUnicode(val) = err {
                panic!("the value of environment variable {var:?} is not valid unicode: {val:?}");
            }
        })
        .ok()
        .map(|val| {
            println!("cargo:rerun-if-env-changed={var}");
            val
        })
}

pub fn get_with_sel4_prefix_relative_fallback(
    var: &str,
    relative_path_from_fallback: impl AsRef<Path>,
) -> PathBuf {
    try_get_with_sel4_prefix_relative_fallback(var, relative_path_from_fallback)
        .unwrap_or_else(|| panic!("{var} or {SEL4_PREFIX_ENV} must be set"))
}

pub fn try_get_with_sel4_prefix_relative_fallback(
    var: &str,
    relative_path_from_fallback: impl AsRef<Path>,
) -> Option<PathBuf> {
    get_asserting_valid_unicode(var)
        .map(PathBuf::from)
        .or_else(|| get_sel4_prefix().map(|fallback| fallback.join(relative_path_from_fallback)))
        .map(|path| {
            println!("cargo:rerun-if-changed={}", path.display());
            path
        })
}

pub fn get_sel4_prefix() -> Option<PathBuf> {
    get_asserting_valid_unicode(SEL4_PREFIX_ENV).map(PathBuf::from)
}

pub fn get_libsel4_include_dirs() -> impl Iterator<Item = PathBuf> {
    get_asserting_valid_unicode(SEL4_INCLUDE_DIRS_ENV)
        .map(|val| val.split(':').map(PathBuf::from).collect())
        .or_else(|| get_sel4_prefix().map(|sel4_prefix| vec![sel4_prefix.join("libsel4/include")]))
        .unwrap_or_else(|| panic!("{SEL4_INCLUDE_DIRS_ENV} or {SEL4_PREFIX_ENV} must be set"))
        .into_iter()
        .map(|path| {
            println!("cargo:rerun-if-changed={}", path.display());
            path
        })
}

pub fn try_find_in_libsel4_include_dirs(relative_path: impl AsRef<Path>) -> Option<PathBuf> {
    for d in get_libsel4_include_dirs() {
        let path = Path::new(&d).join(relative_path.as_ref());
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn find_in_libsel4_include_dirs(relative_path: impl AsRef<Path>) -> PathBuf {
    let relative_path = relative_path.as_ref();
    try_find_in_libsel4_include_dirs(relative_path).unwrap_or_else(|| {
        panic!(
            "{} not found in libsel4 include path",
            relative_path.display()
        )
    })
}

pub fn try_get_or_find_in_libsel4_include_dirs(
    var: &str,
    relative_path: impl AsRef<Path>,
) -> Option<PathBuf> {
    get_asserting_valid_unicode(var)
        .map(PathBuf::from)
        .or_else(|| try_find_in_libsel4_include_dirs(relative_path))
}

pub fn get_or_find_in_libsel4_include_dirs(var: &str, relative_path: impl AsRef<Path>) -> PathBuf {
    let relative_path = relative_path.as_ref();
    try_get_or_find_in_libsel4_include_dirs(var, relative_path).unwrap_or_else(|| {
        panic!(
            "{} not in env and {} not found in libsel4 include path",
            var,
            relative_path.display(),
        )
    })
}
