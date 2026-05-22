//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(dead_code)]

use core::fmt;

pub struct D;

impl fmt::Write for D {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            putc(c);
        }
        Ok(())
    }
}

fn putc(c: u8) {
    semihosting::sys::arm_compat::sys_writec(c);
}
