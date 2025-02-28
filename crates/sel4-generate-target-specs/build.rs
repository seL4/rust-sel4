//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let has_metadata_key = "target_spec_has_metadata";
    let has_is_builtin_key = "target_spec_has_is_builtin";
    let target_tuple_key = "target_tuple";
    let keys = &[has_metadata_key, has_is_builtin_key, target_tuple_key];
    if rustversion::cfg!(any(
        all(not(nightly), since(1.80)),
        all(nightly, since(2024 - 05 - 05))
    )) {
        for key in keys {
            println!("cargo:rustc-check-cfg=cfg({key})");
        }
    }
    if rustversion::cfg!(any(
        all(not(nightly), since(1.78)),
        all(nightly, since(2024 - 03 - 15))
    )) {
        println!("cargo:rustc-cfg={has_metadata_key}");
    }
    if rustversion::cfg!(any(
        all(not(nightly), before(1.84)),
        all(nightly, before(2024 - 11 - 04))
    )) {
        println!("cargo:rustc-cfg={has_is_builtin_key}");
    }
    if rustversion::cfg!(any(
        all(not(nightly), since(1.84)),
        all(nightly, since(2024 - 11 - 02))
    )) {
        println!("cargo:rustc-cfg={target_tuple_key}");
    }
}
