//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::path::Path;
use std::process::Stdio;
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
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    let status = &output.status;

    if !status.success() {
        panic!("{} failed with {}", tool.path().display(), status);
    };

    let lib_path = Path::new(str::from_utf8(&output.stdout).unwrap())
        .parent()
        .unwrap();

    assert!(lib_path.has_root());

    println!("cargo:rustc-link-search=native={}", lib_path.display());
}
