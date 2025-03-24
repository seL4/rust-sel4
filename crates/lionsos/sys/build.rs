//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env::{self, VarError};
use std::path::PathBuf;

pub const SEL4_INCLUDE_DIRS_ENV: &str = "DEP_SEL4_INCLUDE";
pub const SDDF_INCLUDE_DIRS_ENV: &str = "DEP_SDDF_INCLUDE";

pub const LIONSOS_INCLUDE_DIRS_ENV: &str = "LIONSOS_INCLUDE_DIRS";

pub fn get_libsel4_include_dirs() -> impl Iterator<Item = PathBuf> {
    get_asserting_valid_unicode(SEL4_INCLUDE_DIRS_ENV)
        .map(|val| val.split(':').map(PathBuf::from).collect::<Vec<_>>())
        .unwrap_or_else(|| panic!("{SEL4_INCLUDE_DIRS_ENV} must be set"))
        .into_iter()
        .inspect(|path| {
            println!("cargo::rerun-if-changed={}", path.display());
        })
}

pub fn get_libsddf_include_dirs() -> impl Iterator<Item = PathBuf> {
    get_asserting_valid_unicode(SDDF_INCLUDE_DIRS_ENV)
        .map(|val| val.split(':').map(PathBuf::from).collect::<Vec<_>>())
        .unwrap_or_else(|| panic!("{SDDF_INCLUDE_DIRS_ENV} must be set"))
        .into_iter()
        .inspect(|path| {
            println!("cargo::rerun-if-changed={}", path.display());
        })
}

pub fn get_liblions_include_dirs() -> impl Iterator<Item = PathBuf> {
    get_asserting_valid_unicode(LIONSOS_INCLUDE_DIRS_ENV)
        .map(|val| val.split(':').map(PathBuf::from).collect::<Vec<_>>())
        .unwrap_or_else(|| panic!("{LIONSOS_INCLUDE_DIRS_ENV} must be set"))
        .into_iter()
        .inspect(|path| {
            println!("cargo::rerun-if-changed={}", path.display());
        })
}

fn get_asserting_valid_unicode(var: &str) -> Option<String> {
    env::var(var)
        .map_err(|err| {
            if let VarError::NotUnicode(val) = err {
                panic!("the value of environment variable {var:?} is not valid unicode: {val:?}");
            }
        })
        .ok()
        .inspect(|_| {
            println!("cargo::rerun-if-env-changed={var}");
        })
}

const HEADER_CONTENTS: &str = r#"
    #include <lions/fs/config.h>
    #include <lions/fs/protocol.h>
"#;

#[rustfmt::skip]
const ALLOWLIST: &[&str] = &[
    "fs_.*",
    "FS_.*",
    "LIONS_FS_.*",
];

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let clang_target = match target_arch.as_str() {
        "aarch64" => "aarch64-unknown-none",
        "arm" => "armv7a-none-eabi",
        "x86_64" => "x86_64-unknown-none-elf",
        arch => arch,
    };

    let mut builder = bindgen::Builder::default()
        .header_contents("wrapper.h", HEADER_CONTENTS)
        .detect_include_paths(false)
        .clang_args(
            get_libsel4_include_dirs()
                .chain(get_libsddf_include_dirs())
                .chain(get_liblions_include_dirs())
                .map(|d| format!("-I{}", d.as_path().display())),
        )
        .clang_arg(format!("--target={clang_target}"))
        .ignore_functions()
        .allowlist_recursively(false);

    for item in ALLOWLIST.iter() {
        builder = builder.allowlist_item(item);
    }

    let bindings = builder
        .flexarray_dst(true)
        .constified_enum_module(".*")
        .derive_eq(true)
        .derive_default(true)
        .generate_comments(false)
        .use_core()
        .generate()
        .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let bindings_path = PathBuf::from(&out_dir).join("bindings.rs");
    bindings.write_to_file(bindings_path).unwrap();

    println!(
        "cargo::metadata=include={}",
        get_liblions_include_dirs()
            .map(|p| p.to_str().unwrap().to_owned())
            .collect::<Vec<_>>()
            .join(":")
    );
}
