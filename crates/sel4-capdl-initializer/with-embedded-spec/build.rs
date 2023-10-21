//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    sel4_capdl_initializer_with_embedded_spec_embedded_spec_validate::run(true);

    // No use in root task.
    // Remove unnecessary alignment gap between segments.
    println!("cargo:rustc-link-arg=--no-rosegment");

    // No external dependencies
    println!("cargo:rerun-if-changed=build.rs");
}
