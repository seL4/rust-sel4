//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let c_files = glob::glob("cbits/*.c")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    cc::Build::new().files(&c_files).compile("cbits");

    for path in &c_files {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
