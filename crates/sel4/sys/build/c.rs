//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::path::Path;

#[rustfmt::skip]
const BLOCKLIST: &[&str] = &[
    "CONFIG_.*",
    "LIBSEL4_MCS_REPLY",
    "__sel4_ipc_buffer",

    ".*_t",

    // generated enums
    "seL4_Syscall_ID",
    ".*invocation_label",

    // deprecated
    "seL4_AsyncEndpointObject",
    "seL4_PageFaultIpcRegisters.*",

    // static checks
    "__type_.*_size_incorrect",
];

const HEADER_CONTENTS: &str = r#"
    #include <sel4/sel4.h>
    #include <sel4/arch/mapping.h>
    #include <sel4/sel4_arch/mapping.h>
"#;

pub fn generate_rust(
    libsel4_include_dirs: impl Iterator<Item = impl AsRef<Path>>,
    extra_blocklist: &[String],
) -> bindgen::Bindings {
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
        .clang_args(libsel4_include_dirs.map(|d| format!("-I{}", d.as_ref().display())))
        .clang_arg(format!("--target={clang_target}"))
        .ignore_functions();

    for item in BLOCKLIST.iter() {
        builder = builder.blocklist_item(item);
    }

    for item in extra_blocklist.iter() {
        builder = builder.blocklist_item(item);
    }

    builder
        .constified_enum_module(".*")
        .derive_eq(true)
        .derive_default(true)
        .generate_comments(false)
        .use_core()
        .generate()
        .unwrap()
}
