//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

// Determine whether rustc includes the following changes:
// - https://blog.rust-lang.org/2024/05/06/check-cfg.html
// - https://github.com/rust-lang/rust/pull/121598
// - https://github.com/rust-lang/rust/pull/126732

fn main() {
    if rustversion::cfg!(any(
        all(not(nightly), since(1.80)),
        all(nightly, since(2024 - 05 - 05))
    )) {
        println!("cargo:rustc-check-cfg=cfg(catch_unwind_intrinsic_so_named)");
        println!("cargo:rustc-check-cfg=cfg(panic_info_message_stable)");
    }
    if rustversion::cfg!(any(
        all(not(nightly), since(1.78)),
        all(nightly, since(2024 - 02 - 28))
    )) {
        println!("cargo:rustc-cfg=catch_unwind_intrinsic_so_named");
    }
    if rustversion::cfg!(any(
        all(not(nightly), since(1.81)),
        all(nightly, since(2024 - 07 - 01))
    )) {
        println!("cargo:rustc-cfg=panic_info_message_stable");
    }
}
