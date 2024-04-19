//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2020 Arm Limited
//
// SPDX-License-Identifier: MIT
//

use core::fmt;

use crate::sys;

/// Corresponds to `seL4_DebugPutChar`.
pub fn debug_put_char(c: u8) {
    sys::seL4_DebugPutChar(c)
}

/// Implements `core::fmt::Write` using [`debug_put_char`].
pub struct DebugWrite;

impl fmt::Write for DebugWrite {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            debug_put_char(c)
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn debug_print_helper(args: fmt::Arguments) {
    fmt::write(&mut DebugWrite, args).unwrap_or_else(|err| {
        panic!("write error: {:?}", err)
    })
}

/// Prints using `seL4_DebugPutChar`.
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::_private::printing::debug_print_helper(format_args!($($arg)*)));
}

/// Prints using `seL4_DebugPutChar`, with a newline.
#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_println!(""));
    ($($arg:tt)*) => ($crate::debug_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub mod _private {
    pub use super::debug_print_helper;
}
