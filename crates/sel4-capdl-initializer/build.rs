//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    // No use in root task.
    // Remove unnecessary alignment gap between segments.
    println!("cargo:rustc-link-arg=--no-rosegment");

    // No external dependencies
    println!("cargo:rerun-if-changed=build.rs");
}
