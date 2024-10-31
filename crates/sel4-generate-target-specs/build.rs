//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let key = "target_spec_has_metadata";
    if rustversion::cfg!(any(
        all(not(nightly), since(1.80)),
        all(nightly, since(2024 - 05 - 05))
    )) {
        println!("cargo:rustc-check-cfg=cfg({key})");
    }
    if rustversion::cfg!(any(
        all(not(nightly), since(1.78)),
        all(nightly, since(2024 - 03 - 15))
    )) {
        println!("cargo:rustc-cfg={key}");
    }
}
