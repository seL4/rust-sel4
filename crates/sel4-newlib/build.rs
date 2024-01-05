//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![feature(exit_status_error)]

use std::path::Path;
use std::str;

fn main() {
    println!("cargo:rustc-link-lib=static=c");
    if cfg!(feature = "nosys") {
        println!("cargo:rustc-link-lib=static=nosys");
    }
    #[cfg(feature = "detect-libc")]
    {
        detect_libc();
    }
}

#[cfg(feature = "detect-libc")]
fn detect_libc() {
    let tool = cc::Build::new().get_compiler();

    assert!(tool.is_like_gnu() || tool.is_like_clang());

    let output = tool
        .to_command()
        .arg("--print-file-name=libc.a")
        .output()
        .unwrap();

    output.status.exit_ok().unwrap();

    let lib_path = Path::new(str::from_utf8(&output.stdout).unwrap())
        .parent()
        .unwrap();

    assert!(lib_path.has_root());

    println!("cargo:rustc-link-search=native={}", lib_path.display());
}
