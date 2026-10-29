//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let before_crash_key = "before_crash";

    println!("cargo::rustc-check-cfg=cfg({before_crash_key})");

    if rustversion::cfg!(any(
        all(not(nightly), before(1.101)),
        all(nightly, before(2026 - 10 - 03))
    )) {
        println!("cargo::rustc-cfg={before_crash_key}");
    }
}
