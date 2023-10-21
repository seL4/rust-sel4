//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_char;
use core::fmt::{self, Write};
use core::panic::PanicInfo;

use sel4_sys::seL4_DebugPutChar;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let _ = writeln!(Debug, "{}", info);
    core::intrinsics::abort()
}

struct Debug;

impl Write for Debug {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            seL4_DebugPutChar(c as c_char)
        }
        Ok(())
    }
}
