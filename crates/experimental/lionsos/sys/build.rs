//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env::{self, VarError};
use std::fs;
use std::path::PathBuf;

pub const SEL4_INCLUDE_DIRS_ENV: &str = "SEL4_INCLUDE_DIRS";
pub const SDDF_INCLUDE_DIRS_ENV: &str = "SDDF_INCLUDE_DIRS";
pub const LIONSOS_INCLUDE_DIRS_ENV: &str = "LIONSOS_INCLUDE_DIRS";

const HEADER_CONTENTS: &str = r#"
    #include <lions/fs/config.h>
    #include <lions/fs/protocol.h>
"#;

#[rustfmt::skip]
const ALLOWLIST: &[&str] = &[
    "fs_.*",
    "FS_.*",
    "MAX_OPEN_FILES",

    "fb_.*",
    "FB_.*",
];

#[rustfmt::skip]
const BLOCKLIST: &[&str] = &[
];

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let libsel4_include_dirs = get_dirs(SEL4_INCLUDE_DIRS_ENV);
    let libsddf_include_dirs = get_dirs(SDDF_INCLUDE_DIRS_ENV);
    let liblions_include_dirs = get_dirs(LIONSOS_INCLUDE_DIRS_ENV);

    let include_dirs = [
        libsel4_include_dirs.iter(),
        libsddf_include_dirs.iter(),
        liblions_include_dirs.iter(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let clang_target = match target_arch.as_str() {
        "aarch64" => "aarch64-unknown-none",
        "arm" => "armv7a-none-eabi",
        "x86_64" => "x86_64-unknown-none-elf",
        arch => arch,
    };

    let static_fns_path = PathBuf::from(&out_dir).join("static_fns.c");
    let static_fns_with_header_path = PathBuf::from(&out_dir).join("static_fns_with_header.c");
    let static_fns_with_header_contents = format!(
        "{HEADER_CONTENTS}\n#include \"{}\"\n",
        static_fns_path.display()
    );
    fs::write(
        &static_fns_with_header_path,
        static_fns_with_header_contents,
    )
    .unwrap();

    let mut builder = bindgen::Builder::default()
        .header_contents("wrapper.h", HEADER_CONTENTS)
        .detect_include_paths(false)
        .clang_arg(format!("--target={clang_target}"))
        .clang_args(
            include_dirs
                .iter()
                .map(|d| format!("-I{}", d.as_path().display())),
        )
        .generate_inline_functions(true)
        .wrap_static_fns(true)
        .wrap_static_fns_path(&static_fns_path)
        .allowlist_recursively(false);

    for item in ALLOWLIST.iter() {
        builder = builder.allowlist_item(item);
    }

    for pattern in BLOCKLIST.iter() {
        builder = builder.blocklist_item(pattern);
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

    let bindings_path = PathBuf::from(&out_dir).join("bindings.rs");
    bindings.write_to_file(bindings_path).unwrap();

    println!(
        "cargo::metadata=include={}",
        liblions_include_dirs
            .iter()
            .map(|p| p.to_str().unwrap().to_owned())
            .collect::<Vec<_>>()
            .join(":")
    );

    cc::Build::new()
        .file(&static_fns_with_header_path)
        .includes(include_dirs)
        .flag("-Wno-sign-compare") // TODO
        .flag("-Wno-unused-function") // TODO
        .compile("lionsos");
}

fn get_dirs(var: &str) -> Vec<PathBuf> {
    get_asserting_valid_unicode(var)
        .map(|val| val.split(':').map(PathBuf::from).collect::<Vec<_>>())
        .unwrap_or_else(|| panic!("{var} must be set"))
        .into_iter()
        .inspect(|path| {
            println!("cargo::rerun-if-changed={}", path.display());
        })
        .collect()
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
