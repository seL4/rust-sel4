//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

fn main() {
    let before_const_generic_ordering_key = "before_const_generic_ordering";

    // see https://github.com/rust-lang/rust/commit/de1b999ff6c981475e4491ea2fff1851655587e5
    let before_move_integer_pointer_cast_key = "before_move_integer_pointer_cast";

    let before_volatile_key = "before_volatile";

    let keys = &[
        before_const_generic_ordering_key,
        before_move_integer_pointer_cast_key,
        before_volatile_key,
    ];
    for key in keys {
        println!("cargo::rustc-check-cfg=cfg({key})");
    }

    if rustversion::cfg!(any(
        all(not(nightly), before(1.89)),
        all(nightly, before(2025 - 06 - 9))
    )) {
        println!("cargo::rustc-cfg={before_const_generic_ordering_key}");
    }

    if rustversion::cfg!(any(
        all(not(nightly), before(1.91)),
        all(nightly, before(2025 - 08 - 9))
    )) {
        println!("cargo::rustc-cfg={before_move_integer_pointer_cast_key}");
    }

    if rustversion::cfg!(any(
        all(not(nightly), before(1.00)),
        all(nightly, before(2026 - 08 - 16))
    )) {
        println!("cargo::rustc-cfg={before_volatile_key}");
    }
}
