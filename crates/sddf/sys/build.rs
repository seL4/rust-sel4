//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env::{self, VarError};
use std::path::PathBuf;

use sel4_build_env::get_libsel4_include_dirs;

pub const SDDF_INCLUDE_DIRS_ENV: &str = "SDDF_INCLUDE_DIRS";

pub fn get_sddf_include_dirs() -> impl Iterator<Item = PathBuf> {
    get_asserting_valid_unicode(SDDF_INCLUDE_DIRS_ENV)
        .map(|val| val.split(':').map(PathBuf::from).collect::<Vec<_>>())
        .unwrap_or_else(|| panic!("{SDDF_INCLUDE_DIRS_ENV} must be set"))
        .into_iter()
        .inspect(|path| {
            println!("cargo:rerun-if-changed={}", path.display());
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
            println!("cargo:rerun-if-env-changed={var}");
        })
}

const HEADER_CONTENTS: &str = r#"
    // #include <sddf/benchmark/queue.h>
    // #include <sddf/benchmark/config.h>
    #include <sddf/blk/config.h>
    #include <sddf/blk/queue.h>
    #include <sddf/gpu/events.h>
    #include <sddf/gpu/gpu.h>
    #include <sddf/gpu/queue.h>
    #include <sddf/network/config.h>
    #include <sddf/network/queue.h>
    #include <sddf/serial/config.h>
    #include <sddf/serial/queue.h>
    #include <sddf/timer/config.h>
    #include <sddf/timer/protocol.h>
"#;

#[rustfmt::skip]
const ALLOWLIST: &[&str] = &[
    "blk_.*",
    "BLK_.*",
    "SDDF_BLK_.*",
    "net_.*",
    "gpu_.*",
    "GPU_.*",
    "region_.*",
    "device_.*",
    "DEVICE_.*",
    "serial_.*",
    "SDDF_SERIAL_.*",
    "timer_.*",
    "SDDF_TIMER_.*",
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
                .chain(get_sddf_include_dirs())
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
}
