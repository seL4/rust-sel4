//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let old_intrinsics_key = "old_intrinsics";
    let keys = &[old_intrinsics_key];
    for key in keys {
        println!("cargo::rustc-check-cfg=cfg({key})");
    }
    if rustversion::cfg!(any(
        all(not(nightly), before(1.89)),
        all(nightly, before(2025 - 06 - 9))
    )) {
        println!("cargo::rustc-cfg={old_intrinsics_key}");
    }
}
