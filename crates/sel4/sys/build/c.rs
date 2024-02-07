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
    let mut builder = bindgen::Builder::default()
        .header_contents("wrapper.h", HEADER_CONTENTS)
        .detect_include_paths(false)
        .clang_args(libsel4_include_dirs.map(|d| format!("-I{}", d.as_ref().display())))
        .ignore_functions();

    for item in BLOCKLIST.iter() {
        builder = builder.blocklist_item(item);
    }

    for item in extra_blocklist.iter() {
        builder = builder.blocklist_item(item);
    }

    {
        // TODO remove once https://github.com/rust-lang/rust-bindgen/pull/2751 is released
        let target = env::var("TARGET").unwrap();
        if let Some(rest) = target.strip_prefix("riscv64imac-") {
            builder = builder.clang_arg(format!("--target=riscv64-{}", rest));
        }
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
