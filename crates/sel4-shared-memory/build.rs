//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let before_volatile_key = "before_volatile";

    println!("cargo::rustc-check-cfg=cfg({before_volatile_key})");

    if rustversion::cfg!(any(
        all(not(nightly), before(1.100)),
        all(nightly, before(2026 - 08 - 16))
    )) {
        println!("cargo::rustc-cfg={before_volatile_key}");
    }
}
