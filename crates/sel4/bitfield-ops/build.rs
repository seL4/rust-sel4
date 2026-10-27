//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    println!("cargo::rustc-check-cfg=cfg(kani)");
}
