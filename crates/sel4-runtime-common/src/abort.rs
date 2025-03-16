//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

sel4_panicking_env::register_abort_trap!(trap);

fn trap() -> ! {
    core::intrinsics::abort()
}
