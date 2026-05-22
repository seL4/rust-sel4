//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let linker_script = manifest_dir.join("x.lds");
    println!("cargo::rerun-if-changed={}", linker_script.display());
    println!("cargo::rustc-link-arg=-T{}", linker_script.display());

    println!("cargo::rustc-link-arg=-pie");
    println!("cargo::rustc-link-arg=--no-eh-frame-hdr");
    println!("cargo::rustc-link-arg=--orphan-handling=error");
}
