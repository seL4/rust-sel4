//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let before_never_type_key = "before_never_type";

    println!("cargo::rustc-check-cfg=cfg({before_never_type_key})");

    if rustversion::cfg!(any(
        all(not(nightly), before(1.100)),
        all(nightly, before(2026 - 08 - 24))
    )) {
        println!("cargo::rustc-cfg={before_never_type_key}");
    }
}
