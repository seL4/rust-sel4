//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use sel4_panicking_env::{abort, register_abort_trap, register_debug_put_char};

use syscalls::{Sysno, syscall};

register_debug_put_char!(debug_put_char);

fn debug_put_char(c: u8) {
    let _ = unsafe { syscall!(Sysno::write, 1, &raw const c, 1) };
}

pub fn exit(status: u8) -> ! {
    let r = unsafe { syscall!(Sysno::exit, status) };

    if let Err(err) = r {
        abort!("exit syscall returned error: {}", err)
    }
    unreachable!()
}

register_abort_trap!(exit_abort);

fn exit_abort() -> ! {
    exit(1)
}
