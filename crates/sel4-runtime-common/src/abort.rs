//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_crash_polyfill::crash;

sel4_panicking_env::register_abort_trap! {
    #[linkage = "weak"]
    trap
}

fn trap() -> ! {
    crash()
}
